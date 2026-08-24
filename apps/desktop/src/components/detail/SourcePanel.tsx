import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { color, fade, fontSize, radius } from '../../design';
import { ChevronLeft, ChevronRight, Download, Link2, Magnet, Play } from 'lucide-react';
import { t } from '../../i18n';
import { EP, MS, SS, spinnerStyle } from './detailStyles';
import type { Meta, Stream, Video } from '../../core/types';
import { useDragScroll } from '../../hooks/useDragScroll';
import { streamMagnetLink, enqueueOfflineDownload } from '../../core/engine';
import { buildOfflineDownloadRequest, streamShellPlan } from '../../core/streamLinks';
import { ContextMenu } from '../ui/ContextMenu';
import { formatEpDate } from './EpisodePanel';

export function streamDisplayText(value: string | undefined): string | undefined {
  const text = value
    ?.replace(/\\r\\n|\\n|\\r/g, '\n')
    .replace(/\r\n|\r/g, '\n')
    .trim();
  return text || undefined;
}

export function SourceRow({
  stream,
  onClick,
  meta,
  episode,
}: {
  stream: Stream;
  onClick: () => void;
  meta?: Meta;
  episode?: Video | null;
}) {
  const [hovered, setHovered] = useState(false);
  const [menuPoint, setMenuPoint] = useState<{ x: number; y: number } | null>(null);
  const [linkPlan, setLinkPlan] = useState<{ isTorrent: boolean; sourceLink?: string; downloadLink?: string } | null>(null);
  useEffect(() => {
    let active = true;
    void streamShellPlan(stream).then((plan) => {
      if (active) setLinkPlan(plan);
    });
    return () => {
      active = false;
    };
  }, [stream]);
  const pluginUnavailable = stream.extra?.pluginUnavailable === true;
  const unavailableReason =
    typeof stream.extra?.pluginUnavailableReason === 'string' && stream.extra.pluginUnavailableReason !== 'no_playable_stream'
      ? stream.extra.pluginUnavailableReason
      : t('sources.plugin_no_playable_stream');
  const heading = pluginUnavailable
    ? t('sources.plugin_unavailable')
    : streamDisplayText(stream.name) || streamDisplayText(stream.title) || streamDisplayText(stream.description) || t('player.source');
  const seenLines = new Set<string>();
  const lines = (pluginUnavailable ? [unavailableReason] : [stream.title, stream.description])
    .map(streamDisplayText)
    .filter((value): value is string => {
      if (!value || value === heading || seenLines.has(value)) return false;
      seenLines.add(value);
      return true;
    });

  const isTorrent = linkPlan?.isTorrent === true;
  const sourceLink = linkPlan?.sourceLink;
  const downloadLink = linkPlan?.downloadLink;

  return (
    <>
      <button
        style={{
          ...SS.streamRow,
          background: hovered ? color.surface : color.bg,
          color: color.textPrimary,
          boxShadow: hovered ? `0 0 0 0.125rem ${color.fillStrong}` : 'none',
        }}
        onClick={onClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        onContextMenu={(e) => {
          e.preventDefault();
          setMenuPoint({ x: e.clientX, y: e.clientY });
        }}
      >
        <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: '0.5rem', width: '100%' }}>
          <span style={SS.streamName}>{heading}</span>
          {stream.addonName && <span style={{ ...SS.streamAddon, color: color.textDim, flexShrink: 0 }}>{stream.addonName}</span>}
        </div>
        {lines.map((line, index) => (
          <span key={`${line}:${index}`} style={SS.streamDesc}>
            {line}
          </span>
        ))}
      </button>
      <ContextMenu
        point={menuPoint}
        onClose={() => setMenuPoint(null)}
        items={[
          { icon: <Play size={15} />, label: t('player.play'), onSelect: onClick },
          ...(sourceLink
            ? [
                {
                  icon: <Link2 size={15} />,
                  label: t('sources.copy_stream_link'),
                  onSelect: () => {
                    void navigator.clipboard.writeText(sourceLink);
                  },
                },
              ]
            : []),
          ...(isTorrent
            ? [
                {
                  icon: <Magnet size={15} />,
                  label: t('sources.copy_magnet_link'),
                  onSelect: () => {
                    void streamMagnetLink(stream).then((link) => {
                      if (link) void navigator.clipboard.writeText(link);
                    });
                  },
                },
              ]
            : []),
          ...(meta
            ? [
                {
                  icon: <Download size={15} />,
                  label: t('sources.download_this_video'),
                  onSelect: () => {
                    void enqueueOfflineDownload(buildOfflineDownloadRequest(meta, stream, episode));
                  },
                },
              ]
            : []),
          ...(downloadLink
            ? [
                {
                  icon: <Link2 size={15} />,
                  label: t('sources.copy_video_download_link'),
                  onSelect: () => {
                    void navigator.clipboard.writeText(downloadLink);
                  },
                },
              ]
            : []),
        ]}
      />
    </>
  );
}

function AddonFilterPills({
  addonNames,
  selectedAddon,
  onSelect,
  style,
}: {
  addonNames: string[];
  selectedAddon: string | null;
  onSelect: (addon: string | null) => void;
  style?: React.CSSProperties;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragProps = useDragScroll(scrollRef);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const handleWheel = useCallback((event: React.WheelEvent<HTMLDivElement>) => {
    const el = scrollRef.current;
    if (!el || el.scrollWidth <= el.clientWidth) return;
    if (Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
    el.scrollLeft += event.deltaY;
    event.preventDefault();
  }, []);

  const checkScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    checkScroll();
    el.addEventListener('scroll', checkScroll, { passive: true });
    const ro = new ResizeObserver(checkScroll);
    ro.observe(el);
    return () => {
      el.removeEventListener('scroll', checkScroll);
      ro.disconnect();
    };
  }, [checkScroll, addonNames.length]);

  return (
    <div style={{ position: 'relative', ...style }}>
      {canScrollLeft && (
        <PillScrollArrow direction="left" onClick={() => scrollRef.current?.scrollBy({ left: -160, behavior: 'smooth' })} />
      )}
      <div ref={scrollRef} style={EP.sourcePills} onWheel={handleWheel} {...dragProps}>
        <button
          style={{ ...(selectedAddon === null ? SS.pill : SS.pillMuted), cursor: 'pointer', border: 'none', flexShrink: 0 }}
          onClick={() => onSelect(null)}
        >
          {t('auto.all')}
        </button>
        {addonNames.map((addon) => (
          <button
            key={addon}
            style={{ ...(selectedAddon === addon ? SS.pill : SS.pillMuted), cursor: 'pointer', border: 'none', flexShrink: 0 }}
            onClick={() => onSelect(selectedAddon === addon ? null : addon)}
          >
            {addon}
          </button>
        ))}
      </div>
      {canScrollRight && (
        <PillScrollArrow direction="right" onClick={() => scrollRef.current?.scrollBy({ left: 160, behavior: 'smooth' })} />
      )}
    </div>
  );
}

function PillScrollArrow({ direction, onClick }: { direction: 'left' | 'right'; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const isLeft = direction === 'left';
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        position: 'absolute',
        top: 0,
        bottom: '0.75rem',
        [direction]: 0,
        zIndex: 2,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: '1.625rem',
        border: 'none',
        cursor: 'pointer',
        padding: 0,
        background: hovered ? color.surface : fade.elevated(0.85),
        color: color.textPrimary,
      }}
    >
      {isLeft ? <ChevronLeft size={15} /> : <ChevronRight size={15} />}
    </button>
  );
}

export function MovieSourcePanel({
  meta,
  streams,
  isLoading,
  availableAddons,
  failedAddons,
  playbackFailure,
  streamAddonCount,
  onPlay,
  onAddonChange,
  onClose,
  onRetryFailed,
  onOpenSettings,
}: {
  meta: Meta;
  streams: Stream[];
  isLoading: boolean;
  availableAddons: string[];
  failedAddons?: string[];
  playbackFailure?: string | null;
  streamAddonCount: number;
  onPlay: (stream: Stream) => void;
  onAddonChange?: (addon: string | null) => void;
  onClose?: () => void;
  onRetryFailed?: () => void;
  onOpenSettings?: () => void;
}) {
  const [selectedAddon, setSelectedAddon] = useState<string | null>(null);

  useEffect(() => {
    setSelectedAddon(null);
  }, [meta.id]);

  const addonNames = useMemo(
    () => (availableAddons.length > 0 ? availableAddons : [...new Set(streams.map((s) => s.addonName).filter((n): n is string => !!n))]),
    [availableAddons, streams],
  );

  const visibleStreams = selectedAddon ? streams.filter((stream) => stream.addonName === selectedAddon) : streams;
  const selectAddon = useCallback(
    (addon: string | null) => {
      const next = addon && selectedAddon === addon ? null : addon;
      setSelectedAddon(next);
      onAddonChange?.(next);
    },
    [onAddonChange, selectedAddon],
  );
  const rootStyle: React.CSSProperties = onClose ? { display: 'flex', flexDirection: 'column', overflow: 'hidden', flex: 1 } : EP.panel;

  return (
    <div style={rootStyle}>
      <div
        style={{
          padding: '0.875rem 1rem 0.75rem',
          borderBottom: `1px solid ${color.line}`,
          display: 'flex',
          alignItems: 'flex-start',
          justifyContent: 'space-between',
          gap: '0.75rem',
        }}
      >
        <div>
          {meta.logo ? (
            <img
              src={meta.logo}
              alt=""
              style={{
                display: 'block',
                maxWidth: '8.75rem',
                maxHeight: '2.75rem',
                objectFit: 'contain',
                objectPosition: 'left',
                marginBottom: '0.125rem',
              }}
            />
          ) : (
            <h3 style={{ ...EP.sourceTitle, margin: '0 0 0.125rem' }}>{meta.name ?? meta.id}</h3>
          )}
        </div>
        {onClose && (
          <button style={MS.overlayCloseBtn} onClick={onClose}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
            </svg>
          </button>
        )}
      </div>

      {(isLoading || addonNames.length > 0) && (
        <AddonFilterPills
          addonNames={addonNames}
          selectedAddon={selectedAddon}
          onSelect={selectAddon}
          style={{ padding: '0.625rem 1rem 0', borderBottom: `1px solid ${color.line}`, flexShrink: 0 }}
        />
      )}

      {isLoading && streamAddonCount > addonNames.length && (
        <div style={{ padding: '0.5rem 1rem 0', color: color.textDim, fontSize: fontSize.sm, fontWeight: 600 }}>
          {t('sources.searching_addons', streamAddonCount - addonNames.length)}
        </div>
      )}

      {!isLoading && !!failedAddons?.length && <FailedAddonsNotice count={failedAddons.length} onRetry={onRetryFailed} />}
      {playbackFailure && <PlaybackFailureNotice message={playbackFailure} />}

      <div key={selectedAddon ?? 'all'} style={EP.inlineSources}>
        {isLoading && visibleStreams.length === 0 && (
          <div style={SS.center}>
            <div style={spinnerStyle} />
          </div>
        )}
        {!isLoading && visibleStreams.length === 0 && (
          <SourcesEmptyState hasAddons={availableAddons.length > 0} onOpenSettings={onOpenSettings} onRetry={onRetryFailed} />
        )}
        {visibleStreams.length > 0 && (
          <div style={EP.inlineStreamList}>
            {visibleStreams.map((stream, i) => (
              <SourceRow
                key={`${stream.addonName ?? ''}:${stream.url ?? stream.infoHash ?? i}`}
                stream={stream}
                onClick={() => onPlay(stream)}
                meta={meta}
              />
            ))}
            {isLoading && (
              <div style={{ ...SS.center, padding: '1rem 0' }}>
                <div style={{ ...spinnerStyle, width: '1.25rem', height: '1.25rem' }} />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function SourcesEmptyState({
  hasAddons,
  onOpenSettings,
  onRetry,
}: {
  hasAddons: boolean;
  onOpenSettings?: () => void;
  onRetry?: () => void;
}) {
  const action = hasAddons ? onRetry : onOpenSettings;
  const actionLabel = hasAddons ? t('sources.retry') : t('settings.open_addon_settings');
  return (
    <div style={{ ...SS.center, flexDirection: 'column', alignItems: 'center', textAlign: 'center' }}>
      <p style={SS.emptyText}>{hasAddons ? t('auto.no_sources_found_3019f12c') : t('sources.no_stream_addons')}</p>
      <p style={{ color: color.textDim, fontSize: fontSize.sm, margin: '0.375rem 0 0', textAlign: 'center', maxWidth: '20rem' }}>
        {hasAddons ? t('sources.no_sources_hint') : t('sources.no_stream_addons_hint')}
      </p>
      {action && (
        <button
          onClick={action}
          style={{
            marginTop: '1rem',
            background: color.light,
            color: color.onLight,
            border: 'none',
            borderRadius: radius.md,
            padding: '0.5625rem 1.125rem',
            fontSize: fontSize.base,
            fontWeight: 700,
            cursor: 'pointer',
          }}
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
}

function FailedAddonsNotice({ count, onRetry }: { count: number; onRetry?: () => void }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: '0.75rem',
        padding: '0.5rem 1rem',
        margin: '0.375rem 1rem 0',
        background: color.fill,
        borderRadius: radius.sm,
      }}
    >
      <span style={{ color: color.textBody, fontSize: fontSize.sm, fontWeight: 600 }}>{t('sources.addons_failed', count)}</span>
      {onRetry && (
        <button
          onClick={onRetry}
          style={{
            background: 'none',
            border: `1px solid ${color.lineStrong}`,
            borderRadius: radius.xs,
            color: color.textPrimary,
            fontSize: fontSize.sm,
            fontWeight: 600,
            padding: '0.25rem 0.625rem',
            cursor: 'pointer',
            flexShrink: 0,
          }}
        >
          {t('sources.retry')}
        </button>
      )}
    </div>
  );
}

function PlaybackFailureNotice({ message }: { message: string }) {
  return (
    <div
      style={{
        padding: '0.625rem 1rem',
        margin: '0.375rem 1rem 0',
        background: fade.tint(0.12),
        border: `1px solid ${fade.tint(0.32)}`,
        borderRadius: radius.sm,
      }}
    >
      <div style={{ color: color.error, fontSize: fontSize.sm, fontWeight: 700 }}>{t('player.playback_error_title')}</div>
      <div style={{ color: color.textBody, fontSize: fontSize.sm, lineHeight: 1.4, marginTop: '0.1875rem', whiteSpace: 'pre-wrap' }}>
        {message}
      </div>
    </div>
  );
}

export function InlineSourceList({
  episode,
  meta,
  streams,
  isLoading,
  availableAddons,
  failedAddons,
  playbackFailure,
  streamAddonCount,
  onBack,
  onPlay,
  onAddonChange,
  onRetryFailed,
  onOpenSettings,
}: {
  episode: Video;
  meta: Meta;
  streams: Stream[];
  isLoading: boolean;
  availableAddons: string[];
  failedAddons?: string[];
  playbackFailure?: string | null;
  streamAddonCount: number;
  onBack: () => void;
  onPlay: (stream: Stream) => void;
  onAddonChange?: (addon: string | null) => void;
  onRetryFailed?: () => void;
  onOpenSettings?: () => void;
}) {
  const [selectedAddon, setSelectedAddon] = useState<string | null>(null);

  useEffect(() => {
    setSelectedAddon(null);
  }, [episode.id]);

  const addonNames = useMemo(
    () => (availableAddons.length > 0 ? availableAddons : [...new Set(streams.map((s) => s.addonName).filter((n): n is string => !!n))]),
    [availableAddons, streams],
  );

  const visibleStreams = selectedAddon ? streams.filter((stream) => stream.addonName === selectedAddon) : streams;
  const selectAddon = useCallback(
    (addon: string | null) => {
      const next = addon && selectedAddon === addon ? null : addon;
      setSelectedAddon(next);
      onAddonChange?.(next);
    },
    [onAddonChange, selectedAddon],
  );
  const epNum = episode.episode ?? episode.number ?? '';
  const seasonNum = episode.season ?? 1;
  const epTitle = episode.title?.trim() || episode.name?.trim() || t('format.episode_number', epNum);
  const episodeMeta = [
    `${t('format.season_abbrev')}${seasonNum} ${t('format.episode_abbrev')}${epNum}`,
    episode.released ? formatEpDate(episode.released) : null,
  ]
    .filter(Boolean)
    .join(' · ');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', flex: 1, overflow: 'hidden' }}>
      <div style={{ padding: '0.625rem 1rem 0.875rem', flexShrink: 0 }}>
        <button style={EP.backToEpisodesBtn} onClick={onBack} title={t('auto.episodes')} aria-label={t('auto.episodes')}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
          </svg>
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginTop: '0.5rem' }}>
          {episode.thumbnail && (
            <img src={episode.thumbnail} alt="" style={EP.sourceEpisodeThumb} onError={(e) => (e.currentTarget.style.display = 'none')} />
          )}
          <div style={{ minWidth: 0, flex: 1 }}>
            <p style={EP.sourceKicker}>{meta.name ?? meta.id}</p>
            <h3 style={EP.sourceTitle}>{epTitle}</h3>
            <p style={EP.sourceSubtitle}>{episodeMeta}</p>
          </div>
        </div>
        {episode.overview && <p style={EP.sourceOverview}>{episode.overview}</p>}
      </div>

      {(isLoading || addonNames.length > 0) && (
        <AddonFilterPills
          addonNames={addonNames}
          selectedAddon={selectedAddon}
          onSelect={selectAddon}
          style={{ padding: '0 1rem 0.625rem', borderTop: `1px solid ${color.line}`, paddingTop: '0.625rem', flexShrink: 0 }}
        />
      )}

      {isLoading && streamAddonCount > addonNames.length && (
        <div style={{ padding: '0 1rem 0.5rem', color: color.textDim, fontSize: fontSize.sm, fontWeight: 600 }}>
          {t('sources.searching_addons', streamAddonCount - addonNames.length)}
        </div>
      )}

      {!isLoading && !!failedAddons?.length && <FailedAddonsNotice count={failedAddons.length} onRetry={onRetryFailed} />}
      {playbackFailure && <PlaybackFailureNotice message={playbackFailure} />}

      <div key={selectedAddon ?? 'all'} style={EP.inlineSources}>
        {isLoading && visibleStreams.length === 0 && (
          <div style={SS.center}>
            <div style={spinnerStyle} />
          </div>
        )}
        {!isLoading && visibleStreams.length === 0 && (
          <SourcesEmptyState hasAddons={availableAddons.length > 0} onOpenSettings={onOpenSettings} onRetry={onRetryFailed} />
        )}
        {visibleStreams.length > 0 && (
          <div style={EP.inlineStreamList}>
            {visibleStreams.map((stream, i) => (
              <SourceRow
                key={`${stream.addonName ?? ''}:${stream.url ?? stream.infoHash ?? i}`}
                stream={stream}
                onClick={() => onPlay(stream)}
                meta={meta}
                episode={episode}
              />
            ))}
            {isLoading && (
              <div style={{ ...SS.center, padding: '1rem 0' }}>
                <div style={{ ...spinnerStyle, width: '1.25rem', height: '1.25rem' }} />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
