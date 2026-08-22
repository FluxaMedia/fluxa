import { useEffect, useState } from 'react';
import { InfoTile, SettingsSection } from './SettingsUI';
import { t } from '../../i18n';
import { isBrowserTarget } from '../../platform/browser';
import { IS_WEBOS, webosCapabilities } from '../../platform/webos';

type Capability = { label: string; detail?: string };

type DeviceCapabilities = {
  device: string;
  version: string;
  route: string;
  pcm: string;
  sampleRates: string;
  audioSupported: Capability[];
  audioUnsupported: Capability[];
  videoHardware: Capability[];
  videoSoftware: Capability[];
  hdr: Capability[];
  videoUnsupported: Capability[];
  fallback: string;
};

const AUDIO_FORMATS: Array<[string, string]> = [
  ['AAC', 'audio/mp4; codecs="mp4a.40.2"'],
  ['Opus', 'audio/webm; codecs="opus"'],
  ['Vorbis', 'audio/webm; codecs="vorbis"'],
  ['FLAC', 'audio/flac'],
  ['AC3', 'audio/mp4; codecs="ac-3"'],
  ['E-AC3', 'audio/mp4; codecs="ec-3"'],
  ['AC4', 'audio/mp4; codecs="ac-4"'],
];

const VIDEO_FORMATS: Array<[string, string]> = [
  ['H.264 / AVC', 'video/mp4; codecs="avc1.640028"'],
  ['H.265 / HEVC', 'video/mp4; codecs="hvc1.2.4.L153.B0"'],
  ['VP9', 'video/webm; codecs="vp09.00.10.08"'],
  ['AV1', 'video/mp4; codecs="av01.0.08M.08"'],
];

async function probeAudio(contentType: string): Promise<{ supported: boolean; detail: string }> {
  if (!navigator.mediaCapabilities?.decodingInfo) return { supported: false, detail: 'MediaCapabilities unavailable' };
  try {
    const result = await navigator.mediaCapabilities.decodingInfo({
      type: 'file',
      audio: { contentType, channels: '2', bitrate: 192_000, samplerate: 48_000 },
    });
    return {
      supported: result.supported,
      detail: [result.smooth ? 'smooth' : null, result.powerEfficient ? 'power-efficient' : null].filter(Boolean).join(', '),
    };
  } catch {
    return { supported: false, detail: 'probe failed' };
  }
}

async function probeVideo(contentType: string): Promise<{ supported: boolean; detail: string }> {
  if (!navigator.mediaCapabilities?.decodingInfo) return { supported: false, detail: 'MediaCapabilities unavailable' };
  try {
    const result = await navigator.mediaCapabilities.decodingInfo({
      type: 'file',
      video: { contentType, width: 3840, height: 2160, bitrate: 25_000_000, framerate: 24 },
    });
    return {
      supported: result.supported,
      detail: [result.smooth ? 'smooth' : null, result.powerEfficient ? 'hardware-like' : 'software/unknown']
        .filter(Boolean)
        .join(', '),
    };
  } catch {
    return { supported: false, detail: 'probe failed' };
  }
}

async function probeDeviceCapabilities(): Promise<DeviceCapabilities> {
  const audioResults = await Promise.all(AUDIO_FORMATS.map(async ([label, contentType]) => [label, await probeAudio(contentType)] as const));
  const videoResults = await Promise.all(VIDEO_FORMATS.map(async ([label, contentType]) => [label, await probeVideo(contentType)] as const));
  const supportedAudio = audioResults.filter(([, result]) => result.supported).map(([label, result]) => ({ label, detail: result.detail }));
  const unsupportedAudio = audioResults.filter(([, result]) => !result.supported).map(([label, result]) => ({ label, detail: result.detail }));
  const supportedVideo = videoResults.filter(([, result]) => result.supported);
  const unsupportedVideo = videoResults.filter(([, result]) => !result.supported).map(([label, result]) => ({ label, detail: result.detail }));

  let sampleRate = 'System/browser-managed';
  let pcm = 'System/browser-managed';
  try {
    const context = new AudioContext();
    sampleRate = `${context.sampleRate} Hz`;
    pcm = `${context.destination.maxChannelCount} channels`;
    await context.close();
  } catch {
    // Browser policy may deny AudioContext creation before a user gesture.
  }

  const webos = IS_WEBOS ? await webosCapabilities() : null;
  const hdr: Capability[] = [];
  if (webos?.hdr10) hdr.push({ label: 'HDR10' });
  if (webos?.dolbyVision) hdr.push({ label: 'Dolby Vision' });
  if (webos?.dolbyAtmos) supportedAudio.push({ label: 'Dolby Atmos / webOS bitstream', detail: 'device-reported' });

  const videoHardware = supportedVideo.filter(([, result]) => result.detail.includes('hardware-like')).map(([label, result]) => ({ label, detail: result.detail }));
  const videoSoftware = supportedVideo.filter(([, result]) => !result.detail.includes('hardware-like')).map(([label, result]) => ({ label, detail: result.detail }));

  return {
    device: webos?.modelName || navigator.platform || 'Browser runtime',
    version: webos?.sdkVersion ? `webOS SDK ${webos.sdkVersion}` : navigator.userAgent,
    route: isBrowserTarget() ? 'Browser/system-managed output' : 'Native MPV/LibVLC output route',
    pcm,
    sampleRates: sampleRate,
    audioSupported: supportedAudio,
    audioUnsupported: unsupportedAudio,
    videoHardware,
    videoSoftware,
    hdr,
    videoUnsupported: unsupportedVideo,
    fallback: isBrowserTarget()
      ? 'Native browser decode; unsupported streams depend on browser/TV support'
      : 'Passthrough preferred → native PCM → transparent PCM fallback',
  };
}

function CapabilityList({ title, values }: { title: string; values: Capability[] }) {
  return (
    <div style={{ padding: '0.875rem 1.125rem', borderBottom: '1px solid rgba(255,255,255,0.08)' }}>
      <p style={{ margin: 0, fontSize: '0.8rem', fontWeight: 600 }}>{title}</p>
      <p style={{ margin: '0.5rem 0 0', color: 'rgba(255,255,255,0.65)', lineHeight: 1.6 }}>
        {values.length > 0 ? values.map((item) => `${item.label}${item.detail ? ` (${item.detail})` : ''}`).join(' · ') : t('settings.none_detected')}
      </p>
    </div>
  );
}

export function DeviceCapabilitiesSection() {
  const [capabilities, setCapabilities] = useState<DeviceCapabilities | null>(null);

  useEffect(() => {
    let active = true;
    void probeDeviceCapabilities().then((value) => {
      if (active) setCapabilities(value);
    });
    return () => {
      active = false;
    };
  }, []);

  if (!capabilities) return <SettingsSection title={t('settings.device_summary')} subtitle={t('settings.device_capabilities_desc')}><InfoTile title={t('settings.device_name')} value="Detecting…" icon={<span>•</span>} /></SettingsSection>;

  return (
    <>
      <SettingsSection title={t('settings.device_summary')} subtitle={t('settings.device_capabilities_desc')}>
        <InfoTile title={t('settings.device_name')} value={capabilities.device} icon={<span>•</span>} />
        <InfoTile title={t('settings.platform_version')} value={capabilities.version} icon={<span>•</span>} mono />
        <InfoTile title={t('settings.audio_route')} value={capabilities.route} icon={<span>•</span>} />
      </SettingsSection>
      <SettingsSection title={t('settings.audio_capabilities')} subtitle={t('settings.audio_fallback')}>
        <InfoTile title={t('settings.pcm_support')} value={capabilities.pcm} icon={<span>•</span>} />
        <InfoTile title={t('settings.sample_rates')} value={capabilities.sampleRates} icon={<span>•</span>} />
        <CapabilityList title={t('settings.detected_support')} values={capabilities.audioSupported} />
        <CapabilityList title={t('settings.not_detected')} values={capabilities.audioUnsupported} />
        <InfoTile title={t('settings.audio_fallback')} value={capabilities.fallback} icon={<span>•</span>} />
      </SettingsSection>
      <SettingsSection title={t('settings.video_capabilities')} subtitle={t('settings.video_decoders')}>
        <CapabilityList title={t('settings.hardware_decoders')} values={capabilities.videoHardware} />
        <CapabilityList title={t('settings.software_decoders')} values={capabilities.videoSoftware} />
        <CapabilityList title={t('settings.hdr_output')} values={capabilities.hdr} />
        <CapabilityList title={t('settings.not_detected')} values={capabilities.videoUnsupported} />
      </SettingsSection>
    </>
  );
}
