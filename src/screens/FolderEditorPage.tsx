import { useMemo, useState } from 'react';
import { ChevronLeft as ArrowBack } from 'lucide-react';
import { effectiveCatalogId, effectiveCatalogType } from '../core/collections';
import type { CatalogSource, HomeCategory, UserCollectionFolder } from '../core/types';
import { t } from '../i18n';
import { contrastOn, cleanUrl, FieldInput, ImagePreviewField, SaveButton, SectionLabel, Card, Chip } from './CollectionEditorPrimitives';

interface Props {
  initial: UserCollectionFolder;
  accent: string;
  catalogOptions: HomeCategory[];
  onDismiss: () => void;
  onSave: (f: UserCollectionFolder) => void;
}

export function FolderEditorPage({ initial, accent, catalogOptions, onDismiss, onSave }: Props) {
  const [title, setTitle] = useState(initial.title ?? '');
  const [imageUrl, setImageUrl] = useState(initial.imageUrl ?? initial.coverImageUrl ?? '');
  const [focusGifUrl, setFocusGifUrl] = useState(initial.focusGifUrl ?? '');
  const [titleLogoUrl, setTitleLogoUrl] = useState(initial.titleLogoUrl ?? '');
  const [heroBackdropUrl, setHeroBackdropUrl] = useState(initial.heroBackdropUrl ?? '');
  const [shape, setShape] = useState(initial.shape ?? 'poster');
  const [catalogId, setCatalogId] = useState(effectiveCatalogId(initial) ?? '');
  const [genre, setGenre] = useState(initial.genre ?? '');

  const browsableCatalogs = catalogOptions.filter((c) => !c.id.startsWith('cw_') && c.type !== 'collection');
  const selectedCatalog = browsableCatalogs.find((c) => c.id === catalogId);

  const genreOptions = useMemo(() => {
    if (!selectedCatalog) return [];
    return [...new Set(selectedCatalog.items.flatMap((m) => m.genres ?? []))].sort((a, b) => a.localeCompare(b));
  }, [selectedCatalog]);

  const canSave = title.trim().length > 0 && catalogId.trim().length > 0;

  function handleSave() {
    if (!canSave) return;
    const genreSuffix = genre ? ` - ${genre}` : '';
    const catalogTitle = selectedCatalog ? `${selectedCatalog.name}${genreSuffix}` : (initial.catalogTitle ?? title.trim()) + genreSuffix;
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
        padding: '1.5rem 2rem 7.5rem',
        boxSizing: 'border-box',
        display: 'flex',
        justifyContent: 'center',
      }}
    >
      <div style={{ width: '100%', maxWidth: '40rem', display: 'flex', flexDirection: 'column', gap: '1.375rem' }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem',
            paddingBottom: '1.125rem',
            borderBottom: '1px solid rgba(255,255,255,0.08)',
          }}
        >
          <button
            onClick={onDismiss}
            style={{
              width: '2.25rem',
              height: '2.25rem',
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
          <span style={{ color: '#fff', fontSize: '1.375rem', fontWeight: 700, letterSpacing: '-0.025em' }}>{t('library.folder')}</span>
        </div>

        <div>
          <SectionLabel>{t('auto.collection_name')}</SectionLabel>
          <Card>
            <FieldInput value={title} placeholder={t('library.folder_name')} onChange={setTitle} accent={accent} />
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              {(['poster', 'square', 'wide'] as const).map((s) => (
                <button
                  key={s}
                  onClick={() => setShape(s)}
                  style={{
                    flex: 1,
                    height: '2.25rem',
                    border: 'none',
                    borderRadius: '0.5rem',
                    background: shape === s ? accent : 'rgba(255,255,255,0.08)',
                    color: shape === s ? contrastOn(accent) : '#fff',
                    fontWeight: 700,
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                    textTransform: 'capitalize',
                  }}
                >
                  {s.charAt(0).toUpperCase() + s.slice(1)}
                </button>
              ))}
            </div>
          </Card>
        </div>

        <div>
          <SectionLabel>{t('library.catalog')}</SectionLabel>
          <Card>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
              {browsableCatalogs.map((catalog) => (
                <Chip
                  key={catalog.id}
                  label={catalog.name}
                  accent={accent}
                  selected={catalogId === catalog.id}
                  onClick={() => {
                    setCatalogId(catalog.id);
                    setGenre('');
                  }}
                />
              ))}
            </div>
            {genreOptions.length > 0 && (
              <>
                <SectionLabel>{t('auto.genre')}</SectionLabel>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                  {genreOptions.map((g) => (
                    <Chip key={g} label={g} accent={accent} selected={genre === g} onClick={() => setGenre(genre === g ? '' : g)} />
                  ))}
                </div>
              </>
            )}
          </Card>
        </div>

        <div>
          <SectionLabel>{t('settings.advanced')}</SectionLabel>
          <Card>
            <ImagePreviewField label={t('library.folder_image')} value={imageUrl} onChange={setImageUrl} accent={accent} />
            <ImagePreviewField label={t('library.folder_focus_gif')} value={focusGifUrl} onChange={setFocusGifUrl} accent={accent} />
            <ImagePreviewField label={t('library.folder_title_logo')} value={titleLogoUrl} onChange={setTitleLogoUrl} accent={accent} />
            <ImagePreviewField
              label={t('library.folder_hero_backdrop')}
              value={heroBackdropUrl}
              onChange={setHeroBackdropUrl}
              accent={accent}
            />
          </Card>
        </div>

        <SaveButton label={t('library.save_folder')} accent={accent} disabled={!canSave} onClick={handleSave} />
      </div>
    </div>
  );
}
