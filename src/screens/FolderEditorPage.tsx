import { useMemo, useState } from 'react';
import { CaretLeft as ArrowBack, CheckCircle } from '@phosphor-icons/react';
import { effectiveCatalogId, effectiveCatalogType } from '../core/collections';
import type { CatalogSource, HomeCategory, UserCollectionFolder } from '../core/types';
import { t } from '../i18n';
import { contrastOn, cleanUrl, FieldInput, ImagePreviewField, SaveButton } from './CollectionEditorPrimitives';

interface Props {
  initial: UserCollectionFolder;
  accent: string;
  catalogOptions: HomeCategory[];
  onDismiss: () => void;
  onSave: (f: UserCollectionFolder) => void;
}

export function FolderEditorPage({ initial, accent, catalogOptions, onDismiss, onSave }: Props) {
  const [title, setTitle] = useState(initial.title ?? '');
  const [imageUrl, setImageUrl] = useState((initial.imageUrl ?? initial.coverImageUrl) ?? '');
  const [focusGifUrl, setFocusGifUrl] = useState(initial.focusGifUrl ?? '');
  const [titleLogoUrl, setTitleLogoUrl] = useState(initial.titleLogoUrl ?? '');
  const [heroBackdropUrl, setHeroBackdropUrl] = useState(initial.heroBackdropUrl ?? '');
  const [shape, setShape] = useState(initial.shape ?? 'poster');
  const [catalogId, setCatalogId] = useState(effectiveCatalogId(initial) ?? '');
  const [genre, setGenre] = useState(initial.genre ?? '');

  const browsableCatalogs = catalogOptions.filter(
    (c) => !c.id.startsWith('cw_') && c.type !== 'collection',
  );
  const selectedCatalog = browsableCatalogs.find((c) => c.id === catalogId);

  const genreOptions = useMemo(() => {
    if (!selectedCatalog) return [];
    return [...new Set(selectedCatalog.items.flatMap((m) => m.genres ?? []))].sort((a, b) =>
      a.localeCompare(b),
    );
  }, [selectedCatalog]);

  const canSave = title.trim().length > 0 && catalogId.trim().length > 0;

  function handleSave() {
    if (!canSave) return;
    const genreSuffix = genre ? ` - ${genre}` : '';
    const catalogTitle = selectedCatalog
      ? `${selectedCatalog.name}${genreSuffix}`
      : (initial.catalogTitle ?? title.trim()) + genreSuffix;
    const sources: CatalogSource[] = catalogId
      ? [
          {
            catalogId: catalogId.trim(),
            type: selectedCatalog?.type ?? effectiveCatalogType(initial) ?? 'movie',
          },
        ]
      : [];
    onSave({
      ...initial,
      title: title.trim(),
      shape,
      catalogId: catalogId.trim() || undefined,
      catalogTitle,
      genre: genre || undefined,
      catalogSources: sources.length ? sources : undefined,
      coverImageUrl: cleanUrl(imageUrl) ?? initial.coverImageUrl,
      imageUrl: cleanUrl(imageUrl),
      focusGifUrl: cleanUrl(focusGifUrl),
      titleLogoUrl: cleanUrl(titleLogoUrl),
      heroBackdropUrl: cleanUrl(heroBackdropUrl),
    });
  }

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 30,
        background: '#080b12',
        overflowY: 'auto',
        padding: '18px 16px 120px',
        display: 'flex',
        flexDirection: 'column',
        gap: 14,
        boxSizing: 'border-box',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <button
          onClick={onDismiss}
          style={{
            width: 36,
            height: 36,
            borderRadius: '50%',
            border: 'none',
            background: 'rgba(255,255,255,0.06)',
            color: '#fff',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexShrink: 0,
          }}
        >
          <ArrowBack size={20} />
        </button>
        <span style={{ color: '#fff', fontSize: 20, fontWeight: 900, fontFamily: 'sans-serif' }}>
          {t('library.folder')}
        </span>
      </div>

      <FieldInput value={title} placeholder={t('library.folder_name')} onChange={setTitle} accent={accent} />
      <ImagePreviewField label={t('library.folder_image')} value={imageUrl} onChange={setImageUrl} accent={accent} />
      <ImagePreviewField label={t('library.folder_focus_gif')} value={focusGifUrl} onChange={setFocusGifUrl} accent={accent} />
      <ImagePreviewField label={t('library.folder_title_logo')} value={titleLogoUrl} onChange={setTitleLogoUrl} accent={accent} />
      <ImagePreviewField label={t('library.folder_hero_backdrop')} value={heroBackdropUrl} onChange={setHeroBackdropUrl} accent={accent} />

      <div style={{ display: 'flex', gap: 8 }}>
        {(['poster', 'square', 'wide'] as const).map((s) => (
          <button
            key={s}
            onClick={() => setShape(s)}
            style={{
              flex: 1,
              height: 36,
              border: 'none',
              borderRadius: 8,
              background: shape === s ? accent : 'rgba(255,255,255,0.08)',
              color: shape === s ? contrastOn(accent) : '#fff',
              fontWeight: 700,
              fontSize: 12,
              fontFamily: 'sans-serif',
              cursor: 'pointer',
              textTransform: 'capitalize',
            }}
          >
            {s.charAt(0).toUpperCase() + s.slice(1)}
          </button>
        ))}
      </div>

      <div style={{ color: 'rgba(255,255,255,0.8)', fontWeight: 900, fontSize: 13, fontFamily: 'sans-serif' }}>
        {t('library.catalog')}
      </div>
      {browsableCatalogs.map((catalog) => {
        const selected = catalogId === catalog.id;
        const textColor = selected ? contrastOn(accent) : '#fff';
        return (
          <div
            key={catalog.id}
            onClick={() => { setCatalogId(catalog.id); setGenre(''); }}
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '10px 12px',
              borderRadius: 8,
              background: selected ? accent : 'rgba(255,255,255,0.06)',
              cursor: 'pointer',
              gap: 8,
            }}
          >
            <span style={{ flex: 1, color: textColor, fontWeight: 700, fontSize: 13, fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {catalog.name}
            </span>
            {selected && <CheckCircle size={18} color={textColor} />}
          </div>
        );
      })}

      {genreOptions.length > 0 && (
        <>
          <div style={{ color: 'rgba(255,255,255,0.8)', fontWeight: 900, fontSize: 13, fontFamily: 'sans-serif' }}>
            Genre
          </div>
          {genreOptions.map((g) => {
            const selected = genre === g;
            const textColor = selected ? contrastOn(accent) : '#fff';
            return (
              <div
                key={g}
                onClick={() => setGenre(selected ? '' : g)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  padding: '10px 12px',
                  borderRadius: 8,
                  background: selected ? accent : 'rgba(255,255,255,0.06)',
                  cursor: 'pointer',
                  gap: 8,
                }}
              >
                <span style={{ flex: 1, color: textColor, fontWeight: 700, fontSize: 13, fontFamily: 'sans-serif' }}>
                  {g}
                </span>
                {selected && <CheckCircle size={18} color={textColor} />}
              </div>
            );
          })}
        </>
      )}

      <SaveButton label={t('library.save_folder')} accent={accent} disabled={!canSave} onClick={handleSave} />
    </div>
  );
}
