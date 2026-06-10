import React, { useEffect, useMemo, useState } from 'react';
import { MovieCard } from '../components/MovieCard';
import { posterPrefsFromState } from '../core/posterPrefs';
import { appPrefs, prefString } from '../core/appPrefs';
import { effectiveCatalogId, exportCollectionsJson, importCollectionsJson } from '../core/collections';
import { saveProfile } from '../core/profiles';
import type { AppState, HomeCategory, LibraryItem, Meta, UserCollection, UserCollectionFolder, UserProfile } from '../core/types';
import { t } from '../i18n';
import { CategoryGridScreen } from './CategoryGridScreen';
import { CollectionEditorScreen } from './CollectionEditorScreen';
import { CollectionsTab } from '../components/library/CollectionsTab';

type Tab = 'watchlist' | 'history' | 'collections';

const NAV_RAIL_WIDTH = 104;
const PX = 58;

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  activeProfile?: UserProfile | null;
  onProfileUpdated?: (profile: UserProfile) => void;
}

export function LibraryScreen({
  state,
  onDispatch,
  onNavigateDetail,
  onBack,
  activeProfile,
  onProfileUpdated,
}: Props) {
  const [tab, setTab] = useState<Tab>('watchlist');
  const [viewAllFolder, setViewAllFolder] = useState<{ title: string; items: Meta[] } | null>(null);
  const [editingCollection, setEditingCollection] = useState<UserCollection | 'new' | null>(null);
  const library = state.library;

  useEffect(() => {
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
  }, []);

  const watchlist = (library.lastWrite?.watchlist ?? []) as LibraryItem[];
  const history = (library.lastWrite?.history ?? []) as LibraryItem[];
  const posterPrefs = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);
  const prefs = useMemo(() => appPrefs(state), [state.settings?.values]);
  const accent = prefString(prefs, 'accentColorArgb', '#FFFFFF');

  const collections: UserCollection[] = activeProfile?.libraryCollections ?? [];
  const homeCategories: HomeCategory[] = state.home.categories ?? [];

  function getItemsForFolder(folder: UserCollectionFolder): Meta[] {
    const catId = effectiveCatalogId(folder);
    if (!catId) return [];
    const cat = homeCategories.find((c) => c.id === catId || c.catalogId === catId);
    if (!cat) return [];
    if (!folder.genre) return cat.items;
    return cat.items.filter((m) => m.genres?.some((g) => g.toLowerCase() === folder.genre!.toLowerCase()));
  }

  async function saveCollections(next: UserCollection[]) {
    if (!activeProfile) return;
    const updated: UserProfile = { ...activeProfile, libraryCollections: next };
    await saveProfile(updated);
    onProfileUpdated?.(updated);
  }

  async function handleSaveCollection(col: UserCollection) {
    const existing = collections.findIndex((c) => c.id === col.id);
    const next = existing >= 0
      ? collections.map((c) => (c.id === col.id ? col : c))
      : [...collections, col];
    await saveCollections(next);
    setEditingCollection(null);
  }

  async function handleDeleteCollection(id: string) {
    await saveCollections(collections.filter((c) => c.id !== id));
  }

  async function handleImportJson(json: string) {
    const imported = importCollectionsJson(json);
    if (!imported.length) return;
    const existingIds = new Set(collections.map((c) => c.id));
    const merged = [...collections, ...imported.filter((c) => !existingIds.has(c.id))];
    await saveCollections(merged);
    setEditingCollection(null);
  }

  function handleExportAll() {
    const json = exportCollectionsJson(collections);
    void navigator.clipboard.writeText(json);
  }

  if (viewAllFolder) {
    return (
      <CategoryGridScreen
        title={viewAllFolder.title}
        items={viewAllFolder.items}
        posterPrefs={posterPrefs}
        onNavigateDetail={onNavigateDetail}
        onBack={() => setViewAllFolder(null)}
        onDispatch={onDispatch}
      />
    );
  }

  if (editingCollection !== null) {
    const initial = editingCollection === 'new' ? null : editingCollection;
    return (
      <div style={{ position: 'relative', height: '100%', paddingLeft: NAV_RAIL_WIDTH, background: '#040508', boxSizing: 'border-box' }}>
        <CollectionEditorScreen
          accent={accent}
          initial={initial}
          allCollections={collections}
          catalogOptions={homeCategories}
          onDismiss={() => setEditingCollection(null)}
          onSave={(c) => void handleSaveCollection(c)}
          onImportJson={(json) => void handleImportJson(json)}
          onExportAll={handleExportAll}
        />
      </div>
    );
  }

  const items = tab === 'watchlist' ? watchlist : history;

  return (
    <div style={styles.screen}>
      <div style={styles.header}>
        <CircleBtn onClick={onBack} size={48}>
          <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
          </svg>
        </CircleBtn>
        <div>
          <p style={styles.title}>{t('auto.my_library_a6c93797')}</p>
          <p style={styles.subtitle}>{t('auto.movies_and_shows_you_saved_to_watch_later')}</p>
        </div>
      </div>

      <div style={styles.tabRow}>
        <TabChip active={tab === 'watchlist'} onClick={() => setTab('watchlist')}>
          {t('auto.my_list')}{watchlist.length > 0 ? ` (${watchlist.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'history'} onClick={() => setTab('history')}>
          {t('auto.continue_watching')}{history.length > 0 ? ` (${history.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'collections'} onClick={() => setTab('collections')}>
          {t('auto.new_collection').replace('New ', '')}{collections.length > 0 ? ` (${collections.length})` : ''}
        </TabChip>
      </div>

      <div style={{ height: 8 }} />

      {tab === 'collections' ? (
        <CollectionsTab
          collections={collections}
          accent={accent}
          onFolderClick={(folder, folderTitle) => setViewAllFolder({ title: folderTitle, items: getItemsForFolder(folder) })}
          onEditCollection={(col) => setEditingCollection(col)}
          onDeleteCollection={(id) => void handleDeleteCollection(id)}
          onNewCollection={() => setEditingCollection('new')}
          onShowAllOnHome={() => void saveCollections(collections.map((c) => ({ ...c, showOnHome: true })))}
        />
      ) : items.length === 0 ? (
        <div style={styles.empty}>
          <p style={styles.emptyTitle}>
            {tab === 'watchlist' ? t('library.your_list_empty') : t('library.nothing_to_continue')}
          </p>
          <p style={styles.emptyHint}>
            {tab === 'watchlist' ? t('library.add_titles_hint') : t('library.start_watching_hint')}
          </p>
        </div>
      ) : (
        <div style={styles.grid}>
          {items.map((item) => (
            <MovieCard
              key={item.id}
              meta={item as unknown as Meta}
              width={posterPrefs.width}
              height={posterPrefs.height}
              radius={posterPrefs.radius}
              layout={posterPrefs.layout}
              hideTitle={posterPrefs.hideTitles}
              onClick={(m) => onNavigateDetail(m)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function CircleBtn({ onClick, size, children }: { onClick: () => void; size: number; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        width: size, height: size, minWidth: size, borderRadius: '50%',
        background: hovered ? 'rgba(255,255,255,0.12)' : 'rgba(255,255,255,0.05)',
        border: 'none', cursor: 'pointer', display: 'flex', alignItems: 'center',
        justifyContent: 'center', flexShrink: 0, transition: 'background 0.15s',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}

function TabChip({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        background: active ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.05)',
        color: active ? '#000000' : '#FFFFFF',
        border: 'none', borderRadius: 20, padding: '8px 20px',
        fontSize: 14, fontWeight: 700, cursor: 'pointer',
        fontFamily: 'sans-serif', transition: 'background 0.15s, color 0.15s',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}

const styles: Record<string, React.CSSProperties> = {
  screen: { background: '#040508', height: '100%', overflowY: 'auto', paddingBottom: 80, paddingLeft: NAV_RAIL_WIDTH },
  header: { display: 'flex', alignItems: 'center', gap: 24, padding: '40px 58px' },
  title: { color: '#FFFFFF', fontSize: 32, fontWeight: 900, margin: '0 0 4px', fontFamily: 'sans-serif', letterSpacing: '2px' },
  subtitle: { color: 'rgba(255,255,255,0.5)', fontSize: 14, margin: 0, fontFamily: 'sans-serif', lineHeight: 1.4 },
  tabRow: { display: 'flex', gap: 10, paddingLeft: PX, paddingRight: PX },
  grid: { display: 'flex', flexWrap: 'wrap', gap: '24px 20px', paddingLeft: PX, paddingRight: PX, paddingBottom: 60 },
  empty: { display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', paddingTop: 80, gap: 10 },
  emptyTitle: { color: '#FFFFFF', fontSize: 20, fontWeight: 700, margin: 0, fontFamily: 'sans-serif' },
  emptyHint: { color: 'rgba(255,255,255,0.4)', fontSize: 14, margin: 0, fontFamily: 'sans-serif', textAlign: 'center', maxWidth: 320, lineHeight: 1.5 },
};
