import { t } from '../../i18n';
import type { EmbeddedMpvStatus, TorrentStats } from '../../core/mpvPlayer';
import { Sparkline } from './PlayerOverlayPrimitives';

interface Props {
  stats: EmbeddedMpvStatus | null;
  torrentStats: TorrentStats | null;
  bufferHistory: number[];
  networkSpeedHistory: number[];
  torrentSpeedHistory: number[];
  stallCount: number;
}

export function PlayerStatsOverlay({
  stats: statsSnap,
  torrentStats: torrentStatsSnap,
  bufferHistory,
  networkSpeedHistory,
  torrentSpeedHistory,
  stallCount,
}: Props) {
  return (
    <div
      style={{
        position: 'fixed',
        top: '3.75rem',
        left: '1.25rem',
        background: 'rgba(0,0,0,0.84)',
        border: '1px solid rgba(255,255,255,0.1)',
        borderRadius: '0.5rem',
        padding: '0.625rem 0.875rem',
        fontFamily: 'monospace',
        fontSize: '0.6875rem',
        color: 'rgba(255,255,255,0.82)',
        lineHeight: 1.8,
        zIndex: 25,
        minWidth: '16.25rem',
        userSelect: 'none',
      }}
    >
      {statsSnap?.width && statsSnap?.height && (
        <div>
          {statsSnap.width}×{statsSnap.height}
          {statsSnap.videoFormat ? `  ${statsSnap.videoFormat}` : ''}
          {statsSnap.fps ? `  ${parseFloat(statsSnap.fps).toFixed(3)} ${t('player.stats_fps')}` : ''}
          {statsSnap.containerFps && statsSnap.fps && Math.abs(parseFloat(statsSnap.containerFps) - parseFloat(statsSnap.fps)) > 0.1
            ? ` (${t('player.stats_container')} ${parseFloat(statsSnap.containerFps).toFixed(3)})`
            : ''}
        </div>
      )}
      {statsSnap?.displayFps && (
        <div>
          {t('player.stats_display_fps')}: {parseFloat(statsSnap.displayFps).toFixed(3)} {t('player.stats_fps')}
        </div>
      )}
      {statsSnap?.hwdecCurrent && statsSnap.hwdecCurrent !== 'no' && statsSnap.hwdecCurrent !== '' && (
        <div>
          {t('player.stats_hwdec')}: {statsSnap.hwdecCurrent}
        </div>
      )}
      {(statsSnap?.colorMatrix || statsSnap?.colorGamma || statsSnap?.colorPrimaries) &&
        (() => {
          const inVals = [statsSnap.colorMatrix, statsSnap.colorGamma, statsSnap.colorPrimaries];
          const outVals = [statsSnap.videoOutMatrix, statsSnap.videoOutGamma, statsSnap.videoOutPrimaries];
          const inStr = inVals.filter(Boolean).join(' / ');
          const outStr = outVals.filter(Boolean).join(' / ');
          const isHdr = statsSnap.sigPeak != null && parseFloat(statsSnap.sigPeak) > 1;
          const colorsDiffer = inStr !== outStr && outStr.length > 0;
          if (colorsDiffer || isHdr) {
            return (
              <>
                <div>
                  {t('player.stats_color_in')}: {inStr}
                  {isHdr ? `  ${t('player.stats_peak')} ${parseFloat(statsSnap.sigPeak!).toFixed(0)}` : ''}
                </div>
                {outStr && (
                  <div>
                    {t('player.stats_color_out')}: {outStr}
                  </div>
                )}
              </>
            );
          }
          return (
            <div>
              {t('player.stats_color')}: {inStr}
            </div>
          );
        })()}
      {(statsSnap?.frameDropCount != null ||
        statsSnap?.decoderFrameDropCount != null ||
        statsSnap?.mistimedFrameCount != null ||
        statsSnap?.voDelayedFrameCount != null) &&
        (() => {
          const vo = parseInt(statsSnap?.frameDropCount ?? '0');
          const dec = parseInt(statsSnap?.decoderFrameDropCount ?? '0');
          const dropStr = vo > 0 || dec > 0 ? `${vo} (vo) ${dec} (dec)` : '0';
          return (
            <div>
              {t('player.stats_dropped')}: {dropStr} {t('player.stats_mistimed')}: {statsSnap?.mistimedFrameCount ?? '0'}{' '}
              {t('player.stats_vo_delayed')}: {statsSnap?.voDelayedFrameCount ?? '0'}
            </div>
          );
        })()}
      {(statsSnap?.videoBitrate || statsSnap?.audioBitrate) && (
        <div>
          {statsSnap.videoBitrate ? `${t('player.stats_video_bitrate')}: ${(parseInt(statsSnap.videoBitrate) / 1000).toFixed(0)} kbps` : ''}
          {statsSnap.audioBitrate
            ? `  ${t('player.stats_audio_bitrate')}: ${(parseInt(statsSnap.audioBitrate) / 1000).toFixed(0)} kbps`
            : ''}
        </div>
      )}
      {(statsSnap?.audioCodec || statsSnap?.audioSamplerate || statsSnap?.audioChannels) && (
        <div>
          {[statsSnap.audioCodec, statsSnap.audioSamplerate ? `${statsSnap.audioSamplerate} Hz` : null, statsSnap.audioChannels]
            .filter(Boolean)
            .join('  ')}
        </div>
      )}
      {statsSnap?.audioOutputMode && <div>{t('player.stats_audio_path')}: {statsSnap.audioOutputMode}</div>}
      {statsSnap?.demuxerCacheDuration != null && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem', flexWrap: 'wrap', rowGap: '0.125rem' }}>
          <span>{t('player.stats_buffer')}:</span>
          <Sparkline data={bufferHistory} gradId="sg-buf" />
          <span>{parseFloat(statsSnap.demuxerCacheDuration ?? '0').toFixed(1)}s</span>
          {stallCount > 0 && (
            <span style={{ color: 'rgba(255,255,255,0.45)' }}>
              {stallCount} {stallCount === 1 ? t('player.stats_stalls') : t('player.stats_stalls_plural')}
            </span>
          )}
          {statsSnap.cacheBufferingState && statsSnap.pausedForCache === 'yes' && (
            <span style={{ color: 'rgba(255,255,255,0.45)' }}>{statsSnap.cacheBufferingState}%</span>
          )}
        </div>
      )}
      {statsSnap?.cacheSpeed != null &&
        (() => {
          const bytes = parseInt(statsSnap.cacheSpeed ?? '0');
          const speedStr = bytes >= 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB/s` : `${(bytes / 1024).toFixed(0)} KB/s`;
          return (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
              <span>{t('player.stats_net')}:</span>
              <Sparkline data={networkSpeedHistory} gradId="sg-net" />
              <span>{speedStr}</span>
            </div>
          );
        })()}
      {statsSnap?.avsync != null && (
        <div>
          {t('player.stats_avsync')}: {parseFloat(statsSnap.avsync).toFixed(3)}s
        </div>
      )}
      {statsSnap?.fileFormat && (
        <div>
          {t('player.stats_container')}: {statsSnap.fileFormat}
          {(() => {
            try {
              const host = new URL(statsSnap.path ?? '').hostname;
              return host && !host.startsWith('127.') ? `  · ${host}` : '';
            } catch {
              return '';
            }
          })()}
        </div>
      )}
      {torrentStatsSnap && torrentStatsSnap.stat >= 2 && (
        <div>
          <div>
            {t('player.stats_torrent')}: {torrentStatsSnap.active_peers}/{torrentStatsSnap.total_peers} {t('player.stats_peers')}{' '}
            {t('player.stats_preload')}: {torrentStatsSnap.preload}%
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
            <span>↓</span>
            <Sparkline data={torrentSpeedHistory} gradId="sg-tor" />
            <span>{(torrentStatsSnap.download_speed / (1024 * 1024)).toFixed(2)} MB/s</span>
          </div>
        </div>
      )}
      <div style={{ marginTop: '0.25rem', borderTop: '1px solid rgba(255,255,255,0.08)', paddingTop: '0.25rem' }}>
        <button
          style={{
            background: 'none',
            border: 'none',
            color: 'rgba(255,255,255,0.5)',
            fontSize: '0.6875rem',
            fontFamily: 'monospace',
            cursor: 'pointer',
            padding: 0,
          }}
          onClick={() => {
            const lines: string[] = [];
            if (statsSnap?.width && statsSnap?.height)
              lines.push(
                `${statsSnap.width}×${statsSnap.height}  ${statsSnap.videoFormat ?? ''}  ${statsSnap.fps ? parseFloat(statsSnap.fps).toFixed(3) + ' fps' : ''}`,
              );
            if (statsSnap?.hwdecCurrent && statsSnap.hwdecCurrent !== 'no') lines.push(`HW: ${statsSnap.hwdecCurrent}`);
            if (statsSnap?.colorMatrix) {
              const inStr = [statsSnap.colorMatrix, statsSnap.colorGamma, statsSnap.colorPrimaries].filter(Boolean).join(' / ');
              const outStr = [statsSnap.videoOutMatrix, statsSnap.videoOutGamma, statsSnap.videoOutPrimaries].filter(Boolean).join(' / ');
              const isHdr = statsSnap.sigPeak != null && parseFloat(statsSnap.sigPeak) > 1;
              if (isHdr || inStr !== outStr) {
                lines.push(`In: ${inStr}${isHdr ? ` peak ${parseFloat(statsSnap.sigPeak!).toFixed(0)}` : ''}`);
                if (outStr) lines.push(`Out: ${outStr}`);
              } else {
                lines.push(`Color: ${inStr}`);
              }
            }
            const voDrop = parseInt(statsSnap?.frameDropCount ?? '0');
            const decDrop = parseInt(statsSnap?.decoderFrameDropCount ?? '0');
            const dropStr = voDrop > 0 || decDrop > 0 ? `${voDrop} (vo) ${decDrop} (dec)` : '0';
            lines.push(
              `Dropped: ${dropStr}  Mistimed: ${statsSnap?.mistimedFrameCount ?? 0}  VO-delay: ${statsSnap?.voDelayedFrameCount ?? 0}`,
            );
            if (statsSnap?.videoBitrate)
              lines.push(
                `Video: ${(parseInt(statsSnap.videoBitrate) / 1000).toFixed(0)} kbps  Audio: ${statsSnap.audioBitrate ? (parseInt(statsSnap.audioBitrate) / 1000).toFixed(0) + ' kbps' : 'n/a'}`,
              );
            if (statsSnap?.audioCodec)
              lines.push(
                [statsSnap.audioCodec, statsSnap.audioSamplerate ? `${statsSnap.audioSamplerate} Hz` : null, statsSnap.audioChannels]
                  .filter(Boolean)
                  .join('  '),
              );
            if (statsSnap?.audioOutputMode) lines.push(`${t('player.stats_audio_path')}: ${statsSnap.audioOutputMode}`);
            if (statsSnap?.cacheSpeed != null) {
              const bytes = parseInt(statsSnap.cacheSpeed ?? '0');
              const speedStr = bytes >= 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB/s` : `${(bytes / 1024).toFixed(0)} KB/s`;
              lines.push(`Net: ${speedStr}`);
            }
            lines.push(`Buffer: ${parseFloat(statsSnap?.demuxerCacheDuration ?? '0').toFixed(1)}s  stalls: ${stallCount}`);
            if (statsSnap?.avsync) lines.push(`A/V: ${parseFloat(statsSnap.avsync).toFixed(3)}s`);
            if (statsSnap?.fileFormat) lines.push(`Container: ${statsSnap.fileFormat}`);
            if (torrentStatsSnap)
              lines.push(
                `Torrent: ${torrentStatsSnap.active_peers}/${torrentStatsSnap.total_peers} peers  ${(torrentStatsSnap.download_speed / (1024 * 1024)).toFixed(2)} MB/s  preload: ${torrentStatsSnap.preload}%`,
              );
            navigator.clipboard.writeText(lines.join('\n')).catch(() => undefined);
          }}
        >
          {t('player.stats_copy')}
        </button>
      </div>
    </div>
  );
}
