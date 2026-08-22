import React, { useCallback, useEffect, useRef, useState } from 'react';
import { color, fade } from '../../design';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { platformOpenExternal } from '../../platform/browser';
import { t } from '../../i18n';
import { S } from './detailStyles';
import type { Trailer } from '../../core/types';

export type TrailerMetadata = Record<string, { title?: string; description?: string; thumbnail?: string }>;

export function youtubeVideoId(url: string): string | null {
  try {
    const parsed = new URL(url);
    if (parsed.hostname.includes('youtu.be')) return parsed.pathname.split('/').filter(Boolean)[0] ?? null;
    if (parsed.hostname.includes('youtube.com'))
      return parsed.searchParams.get('v') || parsed.pathname.split('/').filter(Boolean).pop() || null;
  } catch {
    return null;
  }
  return null;
}

export function youtubeThumbnail(url: string): string | null {
  const id = youtubeVideoId(url);
  return id ? `https://img.youtube.com/vi/${id}/hqdefault.jpg` : null;
}

export async function fetchYoutubeTrailerMetadata(url: string): Promise<TrailerMetadata[string] | null> {
  try {
    const response = await fetch(`https://www.youtube.com/oembed?${new URLSearchParams({ url, format: 'json' }).toString()}`);
    if (!response.ok) return null;
    const data = (await response.json()) as { title?: string; author_name?: string; thumbnail_url?: string };
    return { title: data.title?.trim(), description: data.author_name?.trim(), thumbnail: data.thumbnail_url?.trim() };
  } catch {
    return null;
  }
}

export function TrailerCarousel({ trailers, trailerMetadata }: { trailers: Trailer[]; trailerMetadata: TrailerMetadata }) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const updateScrollState = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    updateScrollState();
    el.addEventListener('scroll', updateScrollState, { passive: true });
    const ro = new ResizeObserver(updateScrollState);
    ro.observe(el);
    return () => {
      el.removeEventListener('scroll', updateScrollState);
      ro.disconnect();
    };
  }, [trailers.length, updateScrollState]);

  return (
    <div style={S.trailerRail}>
      {canScrollLeft && (
        <TrailerScrollButton direction="left" onClick={() => scrollRef.current?.scrollBy({ left: -520, behavior: 'smooth' })} />
      )}
      <div ref={scrollRef} style={S.trailerList}>
        {trailers.map((trailer, index) => (
          <TrailerCard key={`${trailer.url}:${index}`} trailer={trailer} index={index} metadata={trailerMetadata[trailer.url]} />
        ))}
      </div>
      {canScrollRight && (
        <TrailerScrollButton direction="right" onClick={() => scrollRef.current?.scrollBy({ left: 520, behavior: 'smooth' })} />
      )}
    </div>
  );
}

function TrailerScrollButton({ direction, onClick }: { direction: 'left' | 'right'; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const isLeft = direction === 'left';
  return (
    <div style={{ ...S.trailerScrollEdge, ...(isLeft ? S.trailerScrollEdgeLeft : S.trailerScrollEdgeRight) }}>
      <button
        type="button"
        aria-label={isLeft ? 'Previous trailers' : 'Next trailers'}
        style={{ ...S.trailerScrollButton, background: hovered ? color.fillActive : fade.elevated(0.88) }}
        onClick={onClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      >
        {isLeft ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
      </button>
    </div>
  );
}

const TrailerCard = React.memo(function TrailerCard({
  trailer,
  index,
  metadata,
}: {
  trailer: Trailer;
  index: number;
  metadata?: TrailerMetadata[string];
}) {
  const [hovered, setHovered] = useState(false);
  const trailerWithDescription = trailer as Trailer & { description?: string };
  const thumbnail = metadata?.thumbnail || youtubeThumbnail(trailer.url);
  const title = metadata?.title?.trim() || trailer.title?.trim() || `${t('auto.trailer')} ${index + 1}`;
  const description =
    metadata?.description?.trim() ||
    trailerWithDescription.description?.trim() ||
    (trailer.type?.trim() && trailer.type.trim().toLowerCase() !== title.toLowerCase() ? trailer.type.trim() : '');

  return (
    <button
      style={{
        ...S.trailerCard,
        transform: hovered ? 'translateY(-0.125rem)' : 'translateY(0)',
        borderColor: hovered ? color.lineStrong : color.line,
      }}
      onClick={() => platformOpenExternal(trailer.url).catch(() => {})}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <span style={S.trailerThumb}>
        {thumbnail ? (
          <img src={thumbnail} alt={title} style={S.trailerThumbImg} loading="lazy" />
        ) : (
          <span style={S.trailerThumbFallback}>{t('auto.trailer')}</span>
        )}
        <span style={S.trailerOverlay} />
        <span style={S.trailerPlayButton}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z" />
          </svg>
        </span>
      </span>
      <span style={S.trailerCardTitle}>{title}</span>
      {description && <span style={S.trailerCardMeta}>{description}</span>}
    </button>
  );
});
