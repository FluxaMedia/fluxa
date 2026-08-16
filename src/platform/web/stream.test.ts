import { afterEach, describe, expect, it, vi } from 'vitest';

async function withTarget(target: 'web' | 'webos', run: (mod: typeof import('./stream')) => void | Promise<void>) {
  vi.stubEnv('VITE_FLUXA_TARGET', target);
  vi.resetModules();
  const mod = await import('./stream');
  await run(mod);
}

afterEach(() => {
  vi.unstubAllEnvs();
  vi.resetModules();
});

const MKV = 'https://cdn.example/movie.mkv';
const MP4 = 'https://cdn.example/movie.mp4';
const HEADERS = { Referer: 'https://example' };

describe('choosePlaybackUrl on webOS', () => {
  it('never reaches for a transcoder the TV does not have', async () => {
    await withTarget('webos', ({ choosePlaybackUrl }) => {
      for (const source of [MKV, MP4, 'https://cdn.example/movie.avi']) {
        expect(choosePlaybackUrl(source, source, null, undefined).mode).toBe('direct');
        expect(choosePlaybackUrl(source, source, { videoCodec: 'hevc', audioCodec: 'eac3' }, undefined).mode).toBe('direct');
      }
    });
  });

  it('only falls back to the local proxy when headers must be sent', async () => {
    await withTarget('webos', ({ choosePlaybackUrl }) => {
      const choice = choosePlaybackUrl(MKV, MKV, null, HEADERS);
      expect(choice.mode).toBe('proxy');
      expect(choice.url).toContain('/proxy?url=');
    });
  });

  it('skips probing entirely', async () => {
    await withTarget('webos', async ({ probeStream }) => {
      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      expect(await probeStream(MKV)).toBeNull();
      expect(fetchSpy).not.toHaveBeenCalled();
      fetchSpy.mockRestore();
    });
  });

  it('treats containers the TV demuxes natively as directly playable', async () => {
    await withTarget('webos', ({ canDirectPlay }) => {
      expect(canDirectPlay(MKV, 'hevc', 'eac3')).toBe(true);
      expect(canDirectPlay('https://cdn.example/movie.ts', 'h264', 'aac')).toBe(true);
    });
  });
});

describe('choosePlaybackUrl in a browser', () => {
  it('still transcodes what the browser cannot demux', async () => {
    await withTarget('web', ({ choosePlaybackUrl }) => {
      const choice = choosePlaybackUrl(MKV, MKV, { videoCodec: 'hevc', audioCodec: 'eac3' }, undefined);
      expect(choice.mode).toBe('transcode');
      expect(choice.url).toContain('/transcode?');
    });
  });

  it('plays directly when nothing needs help', async () => {
    await withTarget('web', ({ choosePlaybackUrl }) => {
      expect(choosePlaybackUrl(MP4, MP4, null, undefined)).toEqual({ url: MP4, mode: 'direct' });
    });
  });
});
