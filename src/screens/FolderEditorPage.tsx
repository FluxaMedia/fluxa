import { useMemo, useState } from 'react';
import { ChevronLeft as ArrowBack } from 'lucide-react';
import { effectiveCatalogId, effectiveCatalogType } from '../core/collections';
import { loadNuvioCollectionSource } from '../core/collectionSources';
import type {
  HomeCategory,
  NuvioAddonCollectionSource,
  NuvioCollectionSource,
  NuvioRemoteCollectionSource,
  UserCollectionFolder,
} from '../core/types';
import { t } from '../i18n';
import {
  contrastOn,
  cleanUrl,
  FieldInput,
  ImagePreviewField,
  SaveButton,
  SectionLabel,
  Card,
  Chip,
  Toggle,
  MoveButtons,
  moveItem,
} from './CollectionEditorPrimitives';

interface Props {
  initial: UserCollectionFolder;
  accent: string;
  catalogOptions: HomeCategory[];
  onDismiss: () => void;
  onSave: (f: UserCollectionFolder) => void;
}

const TMDB_SOURCE_TYPES = ['LIST', 'COLLECTION', 'COMPANY', 'NETWORK', 'DISCOVER', 'PERSON', 'DIRECTOR'] as const;

function sourceLabel(source: NuvioCollectionSource, catalogOptions: HomeCategory[]): string {
  if (source.provider === 'addon') {
    const catalog = catalogOptions.find((c) => c.id === source.catalogId);
    return `${catalog?.name ?? source.catalogId}${source.genre ? ` · ${source.genre}` : ''}`;
  }
  if (source.provider === 'trakt') return `Trakt · ${source.title || source.traktListId}`;
  return `TMDB ${source.tmdbSourceType ?? 'DISCOVER'}${source.tmdbId ? ` · ${source.tmdbId}` : ''}`;
}

function SourceRow({
  source,
  catalogOptions,
  index,
  total,
  onMove,
  onRemove,
}: {
  source: NuvioCollectionSource;
  catalogOptions: HomeCategory[];
  index: number;
  total: number;
  onMove: (delta: -1 | 1) => void;
  onRemove: () => void;
}) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.625rem',
        padding: '0.5rem 0.625rem',
        background: 'rgba(255,255,255,0.05)',
        borderRadius: '0.5rem',
      }}
    >
      <span
        style={{
          flex: 1,
          minWidth: 0,
          color: '#fff',
          fontSize: '0.8125rem',
          fontWeight: 600,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {sourceLabel(source, catalogOptions)}
      </span>
      <MoveButtons
        onUp={() => onMove(-1)}
        onDown={() => onMove(1)}
        onRemove={onRemove}
        upDisabled={index === 0}
        downDisabled={index === total - 1}
      />
    </div>
  );
}

function AddTmdbSource({ accent, onAdd }: { accent: string; onAdd: (source: NuvioRemoteCollectionSource) => void }) {
  const [sourceType, setSourceType] = useState<(typeof TMDB_SOURCE_TYPES)[number]>('DISCOVER');
  const [tmdbId, setTmdbId] = useState('');
  const [mediaType, setMediaType] = useState<'MOVIE' | 'TV'>('MOVIE');
  const [genres, setGenres] = useState('');
  const [year, setYear] = useState('');
  const [minRating, setMinRating] = useState('');

  const canAdd = sourceType === 'DISCOVER' || tmdbId.trim().length > 0;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.625rem' }}>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
        {TMDB_SOURCE_TYPES.map((s) => (
          <Chip key={s} label={s} accent={accent} selected={sourceType === s} onClick={() => setSourceType(s)} />
        ))}
      </div>
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        {(['MOVIE', 'TV'] as const).map((m) => (
          <button
            key={m}
            onClick={() => setMediaType(m)}
            style={{
              flex: 1,
              height: '2.25rem',
              border: 'none',
              borderRadius: '0.5rem',
              background: mediaType === m ? accent : 'rgba(255,255,255,0.08)',
              color: mediaType === m ? contrastOn(accent) : '#fff',
              fontWeight: 700,
              fontSize: '0.75rem',
              cursor: 'pointer',
            }}
          >
            {m}
          </button>
        ))}
      </div>
      {sourceType === 'DISCOVER' ? (
        <>
          <FieldInput value={genres} placeholder={t('library.folder_tmdb_genre_ids')} onChange={setGenres} accent={accent} />
          <FieldInput value={year} placeholder={t('library.folder_tmdb_year')} onChange={setYear} accent={accent} />
          <FieldInput value={minRating} placeholder={t('library.folder_tmdb_min_rating')} onChange={setMinRating} accent={accent} />
        </>
      ) : (
        <FieldInput value={tmdbId} placeholder={t('library.folder_tmdb_id')} onChange={setTmdbId} accent={accent} />
      )}
      <SaveButton
        label={t('library.folder_add_source')}
        accent={accent}
        disabled={!canAdd}
        onClick={() => {
          const filters =
            sourceType === 'DISCOVER'
              ? {
                  ...(genres.trim() && { withGenres: genres.trim() }),
                  ...(year.trim() && { year: year.trim() }),
                  ...(minRating.trim() && { voteAverageGte: minRating.trim() }),
                }
              : undefined;
          onAdd({
            provider: 'tmdb',
            mediaType,
            tmdbSourceType: sourceType,
            tmdbId: tmdbId.trim() ? Number(tmdbId.trim()) : undefined,
            filters,
          });
          setTmdbId('');
          setGenres('');
          setYear('');
          setMinRating('');
        }}
      />
    </div>
  );
}

function AddTraktSource({ accent, onAdd }: { accent: string; onAdd: (source: NuvioRemoteCollectionSource) => void }) {
  const [listId, setListId] = useState('');
  const [mediaType, setMediaType] = useState<'MOVIE' | 'TV'>('MOVIE');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.625rem' }}>
      <FieldInput value={listId} placeholder={t('library.folder_trakt_list_id')} onChange={setListId} accent={accent} />
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        {(['MOVIE', 'TV'] as const).map((m) => (
          <button
            key={m}
            onClick={() => setMediaType(m)}
            style={{
              flex: 1,
              height: '2.25rem',
              border: 'none',
              borderRadius: '0.5rem',
              background: mediaType === m ? accent : 'rgba(255,255,255,0.08)',
              color: mediaType === m ? contrastOn(accent) : '#fff',
              fontWeight: 700,
              fontSize: '0.75rem',
              cursor: 'pointer',
            }}
          >
            {m}
          </button>
        ))}
      </div>
      <SaveButton
        label={t('library.folder_add_source')}
        accent={accent}
        disabled={!listId.trim() || Number.isNaN(Number(listId.trim()))}
        onClick={() => {
          onAdd({ provider: 'trakt', mediaType, traktListId: Number(listId.trim()), sortBy: 'rank', sortHow: 'asc' });
          setListId('');
        }}
      />
    </div>
  );
}

export function FolderEditorPage({ initial, accent, catalogOptions, onDismiss, onSave }: Props) {
  const [title, setTitle] = useState(initial.title ?? '');
  const [imageUrl, setImageUrl] = useState(initial.imageUrl ?? initial.coverImageUrl ?? '');
  const [focusGifUrl, setFocusGifUrl] = useState(initial.focusGifUrl ?? '');
  const [focusGifEnabled, setFocusGifEnabled] = useState(initial.focusGifEnabled ?? true);
  const [titleLogoUrl, setTitleLogoUrl] = useState(initial.titleLogoUrl ?? '');
  const [heroBackdropUrl, setHeroBackdropUrl] = useState(initial.heroBackdropUrl ?? '');
  const [heroVideoUrl, setHeroVideoUrl] = useState(initial.heroVideoUrl ?? '');
  const [hideTitle, setHideTitle] = useState(initial.hideTitle ?? false);
  const [coverEmoji, setCoverEmoji] = useState(initial.coverEmoji ?? '');
  const [shape, setShape] = useState(initial.shape ?? 'poster');
  const [sources, setSources] = useState<NuvioCollectionSource[]>(() => {
    if (initial.sources?.length) return initial.sources;
    const catalogId = effectiveCatalogId(initial);
    const type = effectiveCatalogType(initial);
    if (!catalogId) return [];
    const catalog = catalogOptions.find((c) => c.id === catalogId);
    return [
      {
        provider: 'addon',
        addonId: catalog?.addonName ?? 'addon',
        catalogId,
        type: type ?? catalog?.type ?? 'movie',
        genre: initial.genre,
      },
    ];
  });
  const [pendingCatalogId, setPendingCatalogId] = useState('');
  const [pendingGenre, setPendingGenre] = useState('');
  const [addingProvider, setAddingProvider] = useState<'tmdb' | 'trakt' | null>(null);

  const browsableCatalogs = catalogOptions.filter((c) => !c.id.startsWith('cw_') && c.type !== 'collection');
  const pendingCatalog = browsableCatalogs.find((c) => c.id === pendingCatalogId);

  const genreOptions = useMemo(() => {
    if (!pendingCatalog) return [];
    return [...new Set(pendingCatalog.items.flatMap((m) => m.genres ?? []))].sort((a, b) => a.localeCompare(b));
  }, [pendingCatalog]);

  const canSave = title.trim().length > 0 && sources.length > 0;

  async function addRemoteSource(source: NuvioRemoteCollectionSource) {
    setSources((prev) => [...prev, source]);
    if (imageUrl.trim() || coverEmoji.trim()) return;
    const items = await loadNuvioCollectionSource(source);
    const cover = items[0]?.poster;
    if (cover) setImageUrl(cover);
  }

  function handleSave() {
    if (!canSave) return;
    const firstAddon = sources.find((s): s is NuvioAddonCollectionSource => s.provider === 'addon');
    onSave({
      ...initial,
      title: title.trim(),
      shape,
      catalogId: firstAddon?.catalogId,
      catalogTitle: firstAddon ? (catalogOptions.find((c) => c.id === firstAddon.catalogId)?.name ?? title.trim()) : initial.catalogTitle,
      genre: firstAddon?.genre,
      catalogSources: firstAddon ? [{ catalogId: firstAddon.catalogId, type: firstAddon.type }] : undefined,
      sources,
      coverImageUrl: cleanUrl(imageUrl) ?? initial.coverImageUrl,
      imageUrl: cleanUrl(imageUrl),
      coverEmoji: coverEmoji.trim() || undefined,
      hideTitle,
      focusGifUrl: cleanUrl(focusGifUrl),
      focusGifEnabled,
      titleLogoUrl: cleanUrl(titleLogoUrl),
      heroBackdropUrl: cleanUrl(heroBackdropUrl),
      heroVideoUrl: cleanUrl(heroVideoUrl),
    });
  }

  function handleImageChange(next: string) {
    setImageUrl(next);
    if (next.trim()) setCoverEmoji('');
  }

  function handleEmojiChange(next: string) {
    setCoverEmoji(next);
    if (next.trim()) setImageUrl('');
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
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <span style={{ flex: 1, color: '#fff', fontWeight: 600, fontSize: '0.8125rem' }}>{t('library.folder_hide_title')}</span>
              <Toggle checked={hideTitle} onChange={setHideTitle} accent={accent} />
            </div>
          </Card>
        </div>

        <div>
          <SectionLabel>{`${t('library.folder_sources')} · ${sources.length}`}</SectionLabel>
          <Card>
            {sources.length === 0 && (
              <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.8125rem', textAlign: 'center', padding: '0.5rem 0' }}>
                {t('library.folder_no_sources_yet')}
              </div>
            )}
            {sources.map((source, index) => (
              <SourceRow
                key={index}
                source={source}
                catalogOptions={catalogOptions}
                index={index}
                total={sources.length}
                onMove={(delta) => setSources((prev) => moveItem(prev, index, delta))}
                onRemove={() => setSources((prev) => prev.filter((_, i) => i !== index))}
              />
            ))}
          </Card>
        </div>

        <div>
          <SectionLabel>{t('library.folder_add_catalog_source')}</SectionLabel>
          <Card>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
              {browsableCatalogs.map((catalog) => (
                <Chip
                  key={catalog.id}
                  label={catalog.name}
                  accent={accent}
                  selected={pendingCatalogId === catalog.id}
                  onClick={() => {
                    setPendingCatalogId(pendingCatalogId === catalog.id ? '' : catalog.id);
                    setPendingGenre('');
                  }}
                />
              ))}
            </div>
            {pendingCatalog && (
              <>
                {genreOptions.length > 0 && (
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                    {genreOptions.map((g) => (
                      <Chip
                        key={g}
                        label={g}
                        accent={accent}
                        selected={pendingGenre === g}
                        onClick={() => setPendingGenre(pendingGenre === g ? '' : g)}
                      />
                    ))}
                  </div>
                )}
                <SaveButton
                  label={t('library.folder_add_source')}
                  accent={accent}
                  onClick={() => {
                    setSources((prev) => [
                      ...prev,
                      {
                        provider: 'addon',
                        addonId: pendingCatalog.addonName ?? 'addon',
                        catalogId: pendingCatalog.id,
                        type: pendingCatalog.type,
                        genre: pendingGenre || undefined,
                      },
                    ]);
                    setPendingCatalogId('');
                    setPendingGenre('');
                  }}
                />
              </>
            )}
          </Card>
        </div>

        <div>
          <SectionLabel>{t('library.folder_add_remote_source')}</SectionLabel>
          <Card>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <Chip
                label="TMDB"
                accent={accent}
                selected={addingProvider === 'tmdb'}
                onClick={() => setAddingProvider(addingProvider === 'tmdb' ? null : 'tmdb')}
              />
              <Chip
                label="Trakt"
                accent={accent}
                selected={addingProvider === 'trakt'}
                onClick={() => setAddingProvider(addingProvider === 'trakt' ? null : 'trakt')}
              />
            </div>
            {addingProvider === 'tmdb' && <AddTmdbSource accent={accent} onAdd={(source) => void addRemoteSource(source)} />}
            {addingProvider === 'trakt' && <AddTraktSource accent={accent} onAdd={(source) => void addRemoteSource(source)} />}
          </Card>
        </div>

        <div>
          <SectionLabel>{t('settings.advanced')}</SectionLabel>
          <Card>
            <ImagePreviewField label={t('library.folder_image')} value={imageUrl} onChange={handleImageChange} accent={accent} />
            <FieldInput value={coverEmoji} placeholder={t('library.folder_cover_emoji')} onChange={handleEmojiChange} accent={accent} />
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <span style={{ flex: 1, color: '#fff', fontWeight: 600, fontSize: '0.8125rem' }}>
                {t('library.folder_focus_gif_enabled')}
              </span>
              <Toggle checked={focusGifEnabled} onChange={setFocusGifEnabled} accent={accent} />
            </div>
            <ImagePreviewField label={t('library.folder_focus_gif')} value={focusGifUrl} onChange={setFocusGifUrl} accent={accent} />
            <ImagePreviewField label={t('library.folder_title_logo')} value={titleLogoUrl} onChange={setTitleLogoUrl} accent={accent} />
            <ImagePreviewField
              label={t('library.folder_hero_backdrop')}
              value={heroBackdropUrl}
              onChange={setHeroBackdropUrl}
              accent={accent}
            />
            <ImagePreviewField label={t('library.folder_hero_video')} value={heroVideoUrl} onChange={setHeroVideoUrl} accent={accent} />
          </Card>
        </div>

        <SaveButton label={t('library.save_folder')} accent={accent} disabled={!canSave} onClick={handleSave} />
      </div>
    </div>
  );
}
