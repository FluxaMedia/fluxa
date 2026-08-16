import { createClient, type RealtimeChannel, type SupabaseClient } from '@supabase/supabase-js';
import { coreInvoke } from './engine';

export interface WatchTogetherContent {
  id: string;
  contentType: string;
  videoId?: string;
  title: string;
}

export interface WatchTogetherMember {
  id: string;
  name: string;
  isHost: boolean;
  buffering: boolean;
}

export interface WatchTogetherState {
  connectionState: 'disconnected' | 'connecting' | 'connected' | 'error';
  roomCode: string | null;
  isHost: boolean;
  members: WatchTogetherMember[];
  content: WatchTogetherContent | null;
  errorMessage: string | null;
}

export type WatchTogetherConnection =
  | { mode: 'websocket'; serverUrl: string; secret?: string }
  | { mode: 'supabase'; projectUrl: string; anonKey: string };

type SupabaseConnection = Extract<WatchTogetherConnection, { mode: 'supabase' }>;
type WebSocketConnection = Extract<WatchTogetherConnection, { mode: 'websocket' }>;

interface WatchRoom {
  code: string;
  host_id: string;
  max_members: number;
  expires_at: string;
}

interface ClientOptions {
  onState: (state: WatchTogetherState) => void;
}

interface DecodedMessage {
  kind: 'room' | 'members' | 'content' | 'sync' | 'pong' | 'error' | 'unknown';
  roomCode: string | null;
  clientId: string | null;
  hostId: string | null;
  members: WatchTogetherMember[];
  content: WatchTogetherContent | null;
  contentMatchesLocal: boolean;
  sequence: number | null;
  positionMs: number | null;
  playing: boolean | null;
  serverTimeMs: number | null;
  clientTimeMs: number | null;
  errorMessage: string | null;
}

interface Transport {
  connect(): Promise<void>;
  send(message: string): void;
  close(): Promise<void>;
}

const websocketHeartbeatMs = 1000;
const supabaseHeartbeatMs = 10000;

const initialState: WatchTogetherState = {
  connectionState: 'disconnected',
  roomCode: null,
  isHost: false,
  members: [],
  content: null,
  errorMessage: null,
};

class WebSocketTransport implements Transport {
  private socket: WebSocket | null = null;
  private onMessage: (message: string) => void;
  private onClose: () => void;

  constructor(private readonly endpoint: string, private readonly secret: string, onMessage: (message: string) => void, onClose: () => void) {
    this.onMessage = onMessage;
    this.onClose = onClose;
  }

  connect() {
    return new Promise<void>((resolve, reject) => {
      const socketUrl = new URL(this.endpoint);
      if (this.secret.trim()) socketUrl.searchParams.set('token', this.secret.trim());
      const socket = new WebSocket(socketUrl.toString());
      this.socket = socket;
      socket.onopen = () => resolve();
      socket.onmessage = (event) => this.onMessage(String(event.data));
      socket.onclose = () => {
        if (this.socket === socket) {
          this.socket = null;
          this.onClose();
        }
      };
      socket.onerror = () => reject(new Error('Watch Together connection failed'));
    });
  }

  send(message: string) {
    if (this.socket?.readyState === WebSocket.OPEN) this.socket.send(message);
  }

  async close() {
    this.socket?.close();
    this.socket = null;
  }
}

interface PresenceMember {
  id: string;
  name: string;
  buffering: boolean;
}

class SupabaseRealtimeTransport implements Transport {
  private channel: RealtimeChannel | null = null;

  constructor(
    private readonly client: SupabaseClient,
    private readonly roomCode: string,
    private readonly userId: string,
    private readonly displayName: string,
    private readonly onMessage: (message: string, senderId: string | null) => void,
    private readonly onMembers: (members: PresenceMember[]) => void,
  ) {}

  async connect() {
    const channel = this.client.channel(`watch-together:${this.roomCode}`, {
      config: { private: true, broadcast: { self: false }, presence: { key: this.userId } },
    });
    this.channel = channel;
    channel.on('broadcast', { event: 'message' }, (event) => {
      const message = event.payload?.message;
      const senderId = event.payload?.senderId;
      if (typeof message === 'string') this.onMessage(message, typeof senderId === 'string' ? senderId : null);
    });
    const updateMembers = () => {
      const members = Object.entries(channel.presenceState()).flatMap(([id, values]) => {
        const value = values[0] as Record<string, unknown> | undefined;
        return value ? [{ id, name: String(value.name ?? 'Guest'), buffering: Boolean(value.buffering) }] : [];
      });
      this.onMembers(members);
    };
    channel.on('presence', { event: 'sync' }, updateMembers);
    channel.on('presence', { event: 'join' }, updateMembers);
    channel.on('presence', { event: 'leave' }, updateMembers);
    await new Promise<void>((resolve, reject) => {
      channel.subscribe((status) => {
        if (status === 'SUBSCRIBED') {
          void channel.track({ name: this.displayName, buffering: false }).then(() => {
            updateMembers();
            resolve();
          }).catch(reject);
        } else if (status === 'CHANNEL_ERROR' || status === 'TIMED_OUT' || status === 'CLOSED') {
          reject(new Error(`Supabase Realtime channel ${status.toLowerCase()}`));
        }
      });
    });
  }

  send(message: string) {
    void this.channel?.send({ type: 'broadcast', event: 'message', payload: { message, senderId: this.userId } });
  }

  async close() {
    if (this.channel) await this.client.removeChannel(this.channel);
    this.channel = null;
  }
}

export class WatchTogetherClient {
  private transport: Transport | null = null;
  private supabase: SupabaseClient | null = null;
  private mode: 'websocket' | 'supabase' = 'websocket';
  private video: HTMLVideoElement | null = null;
  private content: WatchTogetherContent | null = null;
  private clientId: string | null = null;
  private hostId: string | null = null;
  private state: WatchTogetherState = initialState;
  private lastSequence = -1;
  private clockOffsetMs = 0;
  private lastSentAt = 0;
  private minSendIntervalMs = websocketHeartbeatMs;
  private heartbeat: ReturnType<typeof setInterval> | null = null;
  private options: ClientOptions;

  constructor(options: ClientOptions) {
    this.options = options;
  }

  attach(video: HTMLVideoElement, content: WatchTogetherContent) {
    this.video = video;
    this.content = content;
    if (this.state.isHost) void this.sendState(true);
  }

  detach() {
    this.video = null;
  }

  async create(connection: WatchTogetherConnection, displayName: string, secret = '') {
    if (connection.mode === 'websocket') {
      await this.connectWebSocket(connection, secret);
      await this.sendCoreMessage('watchTogetherCreate', { displayName });
      return;
    }
    this.stop();
    const { client, userId } = await this.signIn(connection);
    const room = await this.callRoomFunction(client, 'create_watch_room', { display_name: displayName });
    await this.connectSupabase(client, userId, room, displayName);
    await this.sendState(true);
  }

  async join(connection: WatchTogetherConnection, roomCode: string, displayName: string, secret = '') {
    if (connection.mode === 'websocket') {
      await this.connectWebSocket(connection, secret);
      await this.sendCoreMessage('watchTogetherJoin', { roomCode, displayName });
      return;
    }
    this.stop();
    const { client, userId } = await this.signIn(connection);
    const room = await this.callRoomFunction(client, 'join_watch_room', { code: roomCode, display_name: displayName });
    await this.connectSupabase(client, userId, room, displayName);
  }

  leave() {
    if (!this.transport) {
      this.stop();
      return;
    }
    if (this.mode === 'supabase') {
      const client = this.supabase;
      const code = this.state.roomCode;
      if (client && code) void client.rpc('leave_watch_room', { code }).then(() => this.stop(), () => this.stop());
      else this.stop();
      return;
    }
    if (this.state.roomCode) void this.sendCoreMessage('watchTogetherLeave', {}).finally(() => this.stop());
    else this.stop();
  }

  notifyLocalPlayback() {
    if (this.state.isHost) void this.sendState(true);
  }

  updateContent(content: WatchTogetherContent) {
    this.content = content;
    if (this.state.isHost && this.state.roomCode) void this.sendCoreMessage('watchTogetherContent', { content });
  }

  get currentState() {
    return this.state;
  }

  private async signIn(connection: SupabaseConnection) {
    const projectUrl = connection.projectUrl.trim();
    const anonKey = connection.anonKey.trim();
    if (!/^https:\/\//.test(projectUrl) || !anonKey) throw new Error('Supabase project URL and anon key are required');
    const client = createClient(projectUrl, anonKey, { realtime: { params: { eventsPerSecond: 10 } } });
    const { data, error } = await client.auth.getSession();
    if (error) throw new Error(error.message);
    if (!data.session) throw new Error('Sign in before starting Watch Together');
    await client.realtime.setAuth(data.session.access_token);
    return { client, userId: data.session.user.id };
  }

  private async callRoomFunction(client: SupabaseClient, name: string, args: Record<string, unknown>) {
    const { data, error } = await client.rpc(name, args);
    if (error) throw new Error(error.message);
    const room = (Array.isArray(data) ? data[0] : data) as WatchRoom | null;
    if (!room?.code) throw new Error('Watch Together room unavailable');
    return room;
  }

  private async connectWebSocket(connection: WebSocketConnection, secret: string) {
    this.stop();
    this.mode = 'websocket';
    this.minSendIntervalMs = websocketHeartbeatMs;
    this.setState({ connectionState: 'connecting', errorMessage: null });
    const input = connection.serverUrl.trim().replace(/^http:/, 'ws:').replace(/^https:/, 'wss:').replace(/\/$/, '');
    const endpoint = input.endsWith('/ws') ? input : `${input}/ws`;
    if (!/^wss?:\/\//.test(endpoint)) throw new Error('Invalid Watch Together server URL');
    this.transport = new WebSocketTransport(endpoint, secret, (message) => void this.handleMessage(message, null), () => this.handleClose());
    await this.startTransport(websocketHeartbeatMs);
  }

  private async connectSupabase(client: SupabaseClient, userId: string, room: WatchRoom, displayName: string) {
    this.mode = 'supabase';
    this.minSendIntervalMs = supabaseHeartbeatMs;
    this.setState({ connectionState: 'connecting', errorMessage: null });
    this.supabase = client;
    this.clientId = userId;
    this.hostId = room.host_id;
    this.transport = new SupabaseRealtimeTransport(
      client,
      room.code,
      userId,
      displayName,
      (message, senderId) => void this.handleMessage(message, senderId),
      (members) => this.handlePresence(members),
    );
    await this.startTransport(supabaseHeartbeatMs);
    this.setState({ roomCode: room.code, isHost: userId === room.host_id });
  }

  private async startTransport(heartbeatMs: number) {
    const transport = this.transport;
    if (!transport) throw new Error('Watch Together transport unavailable');
    try {
      await transport.connect();
      this.setState({ connectionState: 'connected' });
      this.heartbeat = setInterval(() => void this.sendHeartbeat(), heartbeatMs);
    } catch (error) {
      await transport.close();
      this.transport = null;
      this.setState({ connectionState: 'error', errorMessage: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }

  private handlePresence(members: PresenceMember[]) {
    const isHost = this.clientId !== null && this.clientId === this.hostId;
    this.setState({
      members: members.map((member) => ({ ...member, isHost: member.id === this.hostId })),
      isHost,
    });
    if (isHost && members.length > 1) void this.sendState(true);
  }

  private async handleMessage(text: string, senderId: string | null) {
    if (this.mode === 'supabase' && this.hostId !== null && senderId !== this.hostId) return;
    const message = await coreInvoke<DecodedMessage>('watchTogetherDecode', JSON.stringify({ text, localContent: this.content }));
    if (!message) return;
    if (message.kind === 'room') {
      this.clientId = message.clientId ?? '';
      this.hostId = message.hostId;
      const isHost = Boolean(message.hostId) && this.clientId === message.hostId;
      this.setState({ connectionState: 'connected', roomCode: message.roomCode ?? '', isHost, members: message.members, errorMessage: null });
      if (isHost) await this.sendState(true);
    } else if (message.kind === 'members') {
      this.hostId = message.hostId;
      this.setState({ isHost: Boolean(message.hostId) && this.clientId === message.hostId, members: message.members });
    } else if (message.kind === 'content') {
      this.setState({ content: message.content });
    } else if (message.kind === 'sync') {
      await this.applySync(message);
    } else if (message.kind === 'pong') {
      const sent = message.clientTimeMs ?? 0;
      const server = message.serverTimeMs ?? 0;
      this.clockOffsetMs = server - (sent + (Date.now() - sent) / 2);
    } else if (message.kind === 'error') {
      this.setState({ connectionState: 'error', errorMessage: message.errorMessage ?? 'Watch Together error' });
    }
  }

  private async applySync(message: DecodedMessage) {
    if (this.state.isHost || !this.video) return;
    if (message.sequence !== null) {
      if (message.sequence <= this.lastSequence) return;
      this.lastSequence = message.sequence;
    }
    if (message.content) this.setState({ content: message.content });
    if (!message.contentMatchesLocal) return;
    const playing = message.playing ?? false;
    const positionMs = message.positionMs ?? 0;
    const serverTimeMs = message.serverTimeMs ?? Date.now();
    const expectedPositionMs = playing ? positionMs + Math.max(0, Date.now() + this.clockOffsetMs - serverTimeMs) : positionMs;
    const correction = await coreInvoke<{ type: string; positionMs?: number; value?: number }>('watchTogetherDriftCorrection', JSON.stringify({ localPositionMs: this.video.currentTime * 1000, expectedPositionMs, hostPlaying: playing, speedCorrectionActive: this.video.playbackRate !== 1 }));
    if (this.video.paused === playing) {
      if (playing) void this.video.play();
      else this.video.pause();
    }
    if (correction?.type === 'seek') this.video.currentTime = Number(correction.positionMs ?? 0) / 1000;
    if (correction?.type === 'speed') this.video.playbackRate = Number(correction.value ?? 1);
    if (correction?.type === 'resetSpeed') this.video.playbackRate = 1;
  }

  private async sendState(force: boolean) {
    if (!this.video || !this.state.isHost || (!force && Date.now() - this.lastSentAt < this.minSendIntervalMs)) return;
    this.lastSentAt = Date.now();
    await this.sendCoreMessage('watchTogetherPlaybackState', { snapshot: { positionMs: Math.round(this.video.currentTime * 1000), durationMs: Number.isFinite(this.video.duration) ? Math.round(this.video.duration * 1000) : 0, isPlaying: !this.video.paused, isBuffering: this.video.readyState < 3 }, content: this.content });
  }

  private async sendHeartbeat() {
    await this.sendState(false);
    if (this.transport instanceof WebSocketTransport) await this.sendCoreMessage('watchTogetherPing', { clientTimeMs: Date.now() });
  }

  private async sendCoreMessage(method: string, args: Record<string, unknown>) {
    const result = await coreInvoke<Record<string, unknown>>(method, JSON.stringify(args));
    if (!result) throw new Error(`Watch Together core method returned no value: ${method}`);
    this.transport?.send(JSON.stringify(result));
  }

  private stop() {
    this.stopHeartbeat();
    void this.transport?.close();
    this.transport = null;
    this.supabase = null;
    this.clientId = null;
    this.hostId = null;
    this.lastSequence = -1;
    this.setState({ ...initialState });
  }

  private handleClose() {
    if (!this.transport) return;
    this.stopHeartbeat();
    this.transport = null;
    this.setState({ ...initialState });
  }

  private stopHeartbeat() {
    if (this.heartbeat) clearInterval(this.heartbeat);
    this.heartbeat = null;
  }

  private setState(next: Partial<WatchTogetherState>) {
    this.state = { ...this.state, ...next };
    this.options.onState(this.state);
  }
}
