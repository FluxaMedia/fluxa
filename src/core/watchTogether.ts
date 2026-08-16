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
  isHost: boolean;
  buffering: boolean;
}

class SupabaseRealtimeTransport implements Transport {
  private client: SupabaseClient;
  private channel: RealtimeChannel | null = null;
  private readonly clientId = crypto.randomUUID();

  constructor(
    private readonly projectUrl: string,
    private readonly anonKey: string,
    private readonly roomCode: string,
    private readonly displayName: string,
    private readonly isHost: boolean,
    private readonly onMessage: (message: string) => void,
    private readonly onMembers: (members: PresenceMember[]) => void,
  ) {
    this.client = createClient(projectUrl, anonKey, { realtime: { params: { eventsPerSecond: 20 } } });
  }

  async connect() {
    const channel = this.client.channel(`watch-together:${this.roomCode}`, { config: { broadcast: { self: false }, presence: { key: this.clientId } } });
    this.channel = channel;
    channel.on('broadcast', { event: 'message' }, (event) => {
      const message = event.payload?.message;
      if (typeof message === 'string') this.onMessage(message);
    });
    const updateMembers = () => {
      const members = Object.entries(channel.presenceState()).flatMap(([id, values]) => {
        const value = values[0] as Record<string, unknown> | undefined;
        return value ? [{ id, name: String(value.name ?? 'Guest'), isHost: Boolean(value.isHost), buffering: Boolean(value.buffering) }] : [];
      });
      this.onMembers(members);
    };
    channel.on('presence', { event: 'sync' }, updateMembers);
    channel.on('presence', { event: 'join' }, updateMembers);
    channel.on('presence', { event: 'leave' }, updateMembers);
    await new Promise<void>((resolve, reject) => {
      channel.subscribe((status) => {
        if (status === 'SUBSCRIBED') {
          void channel.track({ name: this.displayName, isHost: this.isHost, buffering: false }).then(() => {
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
    void this.channel?.send({ type: 'broadcast', event: 'message', payload: { message } });
  }

  async close() {
    if (this.channel) await this.client.removeChannel(this.channel);
    this.channel = null;
  }
}

export class WatchTogetherClient {
  private transport: Transport | null = null;
  private video: HTMLVideoElement | null = null;
  private content: WatchTogetherContent | null = null;
  private clientId: string | null = null;
  private state: WatchTogetherState = initialState;
  private lastSequence = -1;
  private clockOffsetMs = 0;
  private lastSentAt = 0;
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
    const roomCode = connection.mode === 'supabase' ? await this.generateRoomCode() : null;
    await this.connect(connection, roomCode, displayName, true, secret);
    if (connection.mode === 'websocket') await this.sendCoreMessage('watchTogetherCreate', { displayName });
    else {
      this.setState({ roomCode, isHost: true, members: [{ id: this.clientId ?? '', name: displayName, isHost: true, buffering: false }] });
      await this.sendState(true);
    }
  }

  async join(connection: WatchTogetherConnection, roomCode: string, displayName: string, secret = '') {
    await this.connect(connection, roomCode, displayName, false, secret);
    if (connection.mode === 'websocket') await this.sendCoreMessage('watchTogetherJoin', { roomCode, displayName });
    else this.setState({ roomCode, isHost: false });
  }

  leave() {
    if (!this.transport) {
      this.stop();
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

  private async connect(connection: WatchTogetherConnection, roomCode: string | null, displayName: string, isHost: boolean, secret: string) {
    this.stop();
    this.setState({ connectionState: 'connecting', errorMessage: null });
    if (connection.mode === 'websocket') {
      const input = connection.serverUrl.trim().replace(/^http:/, 'ws:').replace(/^https:/, 'wss:').replace(/\/$/, '');
      const endpoint = input.endsWith('/ws') ? input : `${input}/ws`;
      if (!/^wss?:\/\//.test(endpoint)) throw new Error('Invalid Watch Together server URL');
      this.transport = new WebSocketTransport(endpoint, secret, (message) => void this.handleMessage(message), () => this.handleClose());
    } else {
      if (!roomCode) throw new Error('Supabase room code is required');
      if (!/^https:\/\//.test(connection.projectUrl.trim()) || !connection.anonKey.trim()) throw new Error('Supabase project URL and anon key are required');
      this.clientId = crypto.randomUUID();
      this.transport = new SupabaseRealtimeTransport(connection.projectUrl.trim(), connection.anonKey.trim(), roomCode, displayName, isHost, (message) => void this.handleMessage(message), (members) => this.handlePresence(members));
    }
    try {
      await this.transport.connect();
      this.setState({ connectionState: 'connected' });
      this.heartbeat = setInterval(() => void this.sendHeartbeat(), 1000);
    } catch (error) {
      await this.transport.close();
      this.transport = null;
      this.setState({ connectionState: 'error', errorMessage: error instanceof Error ? error.message : String(error) });
      throw error;
    }
  }

  private handlePresence(members: PresenceMember[]) {
    const host = members.find((member) => member.isHost);
    this.setState({ members, isHost: members.some((member) => member.id === this.clientId && member.isHost), roomCode: this.state.roomCode });
    if (host && this.state.isHost && members.length > 1) void this.sendState(true);
  }

  private async handleMessage(text: string) {
    const message = await coreInvoke<DecodedMessage>('watchTogetherDecode', JSON.stringify({ text, localContent: this.content }));
    if (!message) return;
    if (message.kind === 'room') {
      this.clientId = message.clientId ?? '';
      const isHost = Boolean(message.hostId) && this.clientId === message.hostId;
      this.setState({ connectionState: 'connected', roomCode: message.roomCode ?? '', isHost, members: message.members, errorMessage: null });
      if (isHost) await this.sendState(true);
    } else if (message.kind === 'members') {
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
    if (!this.video || !this.state.isHost || (!force && Date.now() - this.lastSentAt < 1000)) return;
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
    this.clientId = null;
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

  private async generateRoomCode() {
    const spec = await coreInvoke<{ alphabet: string; length: number }>('watchTogetherRoomCodeSpec', '{}');
    if (!spec) throw new Error('Watch Together room code spec unavailable');
    const bytes = new Uint8Array(spec.length);
    crypto.getRandomValues(bytes);
    return Array.from(bytes, (byte) => spec.alphabet[byte % spec.alphabet.length]).join('');
  }
}
