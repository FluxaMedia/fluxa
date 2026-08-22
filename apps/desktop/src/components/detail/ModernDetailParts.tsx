import React, { useEffect, useRef, useState } from 'react';
import { Check, ChevronDown } from 'lucide-react';
import { t } from '../../i18n';
import { MS } from './detailStyles';

export function GenreTag({ label, onClick }: { label: string; onClick?: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <span
      style={{ ...MS.genreTag, textDecoration: hovered ? 'underline' : 'none' }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') onClick?.();
      }}
    >
      {label}
    </span>
  );
}

export const similarSourceOptions = ['auto', 'trakt', 'simkl', 'tmdb'] as const;

export function SimilarSourcePicker({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const [open, setOpen] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const closeOnOutsideClick = (event: MouseEvent) => {
      if (!pickerRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };
    document.addEventListener('mousedown', closeOnOutsideClick);
    document.addEventListener('keydown', closeOnEscape);
    return () => {
      document.removeEventListener('mousedown', closeOnOutsideClick);
      document.removeEventListener('keydown', closeOnEscape);
    };
  }, []);

  const labelFor = (source: string) => t(`detail.similar_source_${source}`);

  return (
    <div ref={pickerRef} style={MS.similarSourcePicker}>
      <button
        type="button"
        aria-label={t('detail.similar_source')}
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        style={MS.similarSourceButton}
      >
        <span>{labelFor(value)}</span>
        <ChevronDown
          size={15}
          strokeWidth={2.5}
          style={{ transform: open ? 'rotate(180deg)' : undefined, transition: 'transform 0.16s ease' }}
        />
      </button>
      {open && (
        <div role="menu" style={MS.similarSourceMenu}>
          {similarSourceOptions.map((source) => {
            const selected = source === value;
            return (
              <button
                key={source}
                type="button"
                role="menuitemradio"
                aria-checked={selected}
                onClick={() => {
                  setOpen(false);
                  if (!selected) onChange(source);
                }}
                style={{ ...MS.similarSourceMenuItem, ...(selected ? MS.similarSourceMenuItemActive : {}) }}
              >
                <span>{labelFor(source)}</span>
                {selected && <Check size={15} strokeWidth={3} />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
