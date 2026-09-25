import React, { useEffect, useLayoutEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import { Bookmark, BookmarkCheck, Info, Play } from 'lucide-react';
import { t } from '../i18n';
import type { Meta } from '../core/types';
import { cardImageUrl } from '../core/imageSizes';
import { coreInvoke } from '../core/engine';
import { Button, IconButton, color, fade, fontSize, lineHeight, radius, space, weight, z } from '../design';

const WIDTH = 21.5;
const MARGIN = 0.75;

function rootFontSize(): number {
  return parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
}

export function PosterPreviewCard({
  meta,
  anchor,
  inLibrary,
  onOpenDetail,
  onPlay,
  onToggleLibrary,
  onMouseEnter,
  onMouseLeave,
}: {
  meta: Meta;
  anchor: DOMRect;
  inLibrary?: boolean;
  onOpenDetail: () => void;
  onPlay?: () => void;
  onToggleLibrary?: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
}) {
  const [box, setBox] = useState<{ left: number; top: number } | null>(null);
  const [element, setElement] = useState<HTMLDivElement | null>(null);

  useLayoutEffect(() => {
    if (!element) return;
    const unit = rootFontSize();
    const width = WIDTH * unit;
    const margin = MARGIN * unit;
    const height = element.offsetHeight;
    const rightPosition = anchor.right + margin;
    const leftPosition = anchor.left - width - margin;
    const fitsRight = rightPosition + width <= window.innerWidth - margin;
    const preferredLeft = fitsRight ? rightPosition : leftPosition;
    const preferredTop = anchor.top - height * 0.16;
    const left = Math.min(Math.max(margin, preferredLeft), window.innerWidth - width - margin);
    const top = Math.min(Math.max(margin, preferredTop), window.innerHeight - height - margin);
    setBox({ left, top });
  }, [element, anchor]);

  const record = meta as unknown as Record<string, unknown>;
  const ratings = record.ratings as Record<string, unknown> | undefined;
  const rawRating = meta.imdbRating ?? record.rating ?? ratings?.imdb;
  const rating =
    typeof rawRating === 'number' || typeof rawRating === 'string' ? String(rawRating) : null;
  const directCast = (meta.cast ?? [])
    .map((member) => {
      const value = member as unknown as Record<string, unknown>;
      return typeof value.name === 'string' ? value.name.trim() : '';
    })
    .filter(Boolean);
  const [castNames, setCastNames] = useState<string[]>(directCast.slice(0, 4));

  useEffect(() => {
    if (directCast.length > 0) {
      setCastNames(directCast.slice(0, 4));
      return;
    }
    let active = true;
    void coreInvoke<{ cast?: Array<{ name?: string }> }>('classifyMetaLinks', JSON.stringify(meta.links ?? [])).then((result) => {
      if (!active) return;
      setCastNames((result?.cast ?? []).map((link) => link.name?.trim() ?? '').filter(Boolean).slice(0, 4));
    });
    return () => {
      active = false;
    };
  }, [meta.cast, meta.links]);

  const timeOffset = typeof record.timeOffset === 'number' ? record.timeOffset : 0;
  const duration = typeof record.duration === 'number' ? record.duration : 0;
  const progress = timeOffset > 0 && duration > 0 ? Math.min(99, Math.round((timeOffset / duration) * 100)) : 0;

  const facts = [meta.releaseInfo || (meta.year ? String(meta.year) : ''), meta.runtime, meta.certification].filter(Boolean);
  const backdrop =
    cardImageUrl(meta.background, { kind: 'backdrop', displayWidth: 344, dpr: window.devicePixelRatio }) ||
    cardImageUrl(meta.poster, { displayWidth: 344, dpr: window.devicePixelRatio });

  return createPortal(
    <div
      ref={setElement}
      className="poster-preview"
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      style={{
        position: 'fixed',
        left: box?.left ?? -9999,
        top: box?.top ?? -9999,
        width: `${WIDTH}rem`,
        zIndex: z.dialog,
        background: color.surface,
        border: `1px solid ${color.lineStrong}`,
        borderRadius: radius.lg,
        overflow: 'hidden',
        boxShadow: `0 1rem 3rem ${fade.shade(0.62)}`,
        visibility: box ? 'visible' : 'hidden',
      }}
    >
      <div style={{ position: 'relative', aspectRatio: '16 / 9', background: color.fill }}>
        {backdrop && <img src={backdrop} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} />}
        <div
          style={{
            position: 'absolute',
            inset: 0,
            background: `linear-gradient(to bottom, transparent 40%, ${fade.shade(0.55)} 78%, ${color.surface} 100%)`,
          }}
        />
        {progress > 0 && (
          <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: '0.1875rem', background: fade.shade(0.5) }}>
            <span style={{ display: 'block', height: '100%', width: `${progress}%`, background: color.accent }} />
          </div>
        )}
      </div>

      <div style={{ padding: space[3], display: 'flex', flexDirection: 'column', gap: space[2.5] }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: space[2] }}>
          {onPlay && (
            <IconButton
              size="2.25rem"
              aria-label={t('common.play')}
              title={t('common.play')}
              onClick={onPlay}
              style={{ background: color.light, borderColor: color.light, color: color.onLight }}
            >
              <Play size={15} fill="currentColor" strokeWidth={0} />
            </IconButton>
          )}
          {onToggleLibrary && (
            <IconButton
              size="2.25rem"
              active={inLibrary}
              aria-label={inLibrary ? t('detail.in_library') : t('detail.add_to_library')}
              title={inLibrary ? t('detail.in_library') : t('detail.add_to_library')}
              onClick={onToggleLibrary}
            >
              {inLibrary ? <BookmarkCheck size={15} /> : <Bookmark size={15} />}
            </IconButton>
          )}
          <Button
            size="sm"
            variant="secondary"
            icon={<Info size={14} />}
            onClick={onOpenDetail}
            style={{ marginLeft: 'auto', height: '2.25rem' }}
          >
            {t('detail.preview_more_info')}
          </Button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: space[1] }}>
          <span style={{ fontSize: fontSize.md, fontWeight: weight.bold, color: color.textPrimary }}>{meta.name}</span>
          {facts.length > 0 && (
            <span style={{ fontSize: fontSize.sm, fontWeight: weight.medium, color: color.textMuted }}>{facts.join(' · ')}</span>
          )}
          {rating && (
            <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.375rem', fontSize: fontSize.sm, fontWeight: weight.semibold }}>
              <span style={{ padding: '0.125rem 0.25rem', borderRadius: radius.xs, background: color.imdb, color: color.black, fontSize: fontSize.xs, fontWeight: weight.bold }}>
                IMDb
              </span>
              <span style={{ color: color.textPrimary }}>{rating}</span>
            </span>
          )}
        </div>

        {meta.description && (
          <p
            style={{
              margin: 0,
              fontSize: fontSize.base,
              lineHeight: lineHeight.relaxed,
              color: color.textBody,
              display: '-webkit-box',
              WebkitLineClamp: 3,
              WebkitBoxOrient: 'vertical',
              overflow: 'hidden',
            }}
          >
            {meta.description}
          </p>
        )}

        {castNames.length > 0 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: space[1] }}>
            <span style={{ fontSize: fontSize.sm, fontWeight: weight.bold, color: color.textPrimary }}>{t('auto.cast')}</span>
            <span style={{ fontSize: fontSize.base, lineHeight: lineHeight.normal, color: color.textBody }}>
              {castNames.join(' · ')}
            </span>
          </div>
        )}
      </div>
    </div>,
    document.body,
  );
}
