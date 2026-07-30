import React, { useEffect, useMemo, useState } from 'react';
import { ArrowLeft, CheckSquare2, Search, Square, X } from 'lucide-react';
import { VirtualizedPosterGrid } from '../components/VirtualizedPosterGrid';
import { FilterDropdown } from '../components/FilterDropdown';
import { posterPrefsFromState } from '../core/posterPrefs';
import { appPrefs, prefString } from '../core/appPrefs';
import { getViewPrefs, setViewPref, whenViewPrefsReady } from '../core/viewPrefs';
import type { AppState, HomeCategory, LibraryItem, Meta, UserProfile } from '../core/types';
import { t } from '../i18n';
import { CategoryGridScreen } from './CategoryGridScreen';
import { CollectionEditorScreen } from './CollectionEditorScreen';
import { CollectionsTab } from '../components/library/CollectionsTab';
import { coreInvoke } from '../core/engine';
import { loadProviderLibraries, PROVIDER_LIBRARIES_CHANGED, type LibraryProvider, type ProviderLibrarySnapshot } from '../core/providerLibraries';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { useLibraryBulkSelection } from '../hooks/useLibraryBulkSelection';
import { useLibraryCollections } from '../hooks/useLibraryCollections';
import { NAV_RAIL_WIDTH, PX, styles } from './libraryScreenStyles';
import { CircleBtn, HistoryTimeline, TabChip } from './LibraryScreenParts';

type Tab = 'watchlist' | 'watching' | 'completed' | 'dropped' | 'collections' | 'airing' | 'rated' | 'history';
type LibrarySource = 'local' | LibraryProvider;

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  activeProfile?: UserProfile | null;
  onProfileUpdated?: (profile: UserProfile) => void;
}

export const LibraryScreen = React.memo(function LibraryScreen({
  state,
  onDispatch,
  onNavigateDetail,
  onBack,
  activeProfile,
  onProfileUpdated,
}: Props) {
  const [tab, setTab] = useState<Tab>(() => (getViewPrefs().libraryTab as Tab) ?? 'watchlist');
  const [query, setQuery] = useState('');
  const [sortBy, setSortBy] = useState<'recent' | 'title' | 'rating'>(() => (getViewPrefs().librarySort as 'recent' | 'title' | 'rating') ?? 'recent');
  const [librarySource, setLibrarySource] = useState<LibrarySource>(() => (getViewPrefs().librarySource as LibrarySource) ?? 'local');
  const [providerLibraries, setProviderLibraries] = useState<Partial<Record<LibraryProvider, ProviderLibrarySnapshot>>>({});

  useEffect(() => {
    void whenViewPrefsReady().then(() => {
      const v = getViewPrefs();
      if (v.libraryTab) setTab(v.libraryTab as Tab);
      if (v.librarySort) setSortBy(v.librarySort as 'recent' | 'title' | 'rating');
      if (v.librarySource) setLibrarySource(v.librarySource as LibrarySource);
    });
  }, []);

  const changeTab = (v: Tab) => { setTab(v); setViewPref('libraryTab', v); };
  const changeSort = (v: 'recent' | 'title' | 'rating') => { setSortBy(v); setViewPref('librarySort', v); };
  const changeLibrarySource = (v: LibrarySource) => { setLibrarySource(v); setViewPref('librarySource', v); };

  useEffect(() => {
    if (getViewPrefs().librarySource) return;
    const source = prefString(appPrefs(state), 'integrationLibrarySource', 'local') as LibrarySource;
    if (source !== 'local') changeLibrarySource(source);
  }, [state.settings?.values]);

  useEffect(() => {
    let active = true;
    const refreshProviderLibraries = () => {
      void loadProviderLibraries().then((libraries) => { if (active) setProviderLibraries(libraries); });
    };
    refreshProviderLibraries();
    window.addEventListener(PROVIDER_LIBRARIES_CHANGED, refreshProviderLibraries);
    return () => {
      active = false;
      window.removeEventListener(PROVIDER_LIBRARIES_CHANGED, refreshProviderLibraries);
    };
  }, [activeProfile?.id]);

  useEffect(() => {
    if (librarySource !== 'local' && !providerLibraries[librarySource]) changeLibrarySource('local');
  }, [librarySource, providerLibraries]);

  const TAB_ORDER: Tab[] = ['watchlist', 'watching', 'completed', 'dropped', 'collections', 'airing', 'rated', 'history'];
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) return;
      if (e.code !== 'BracketLeft' && e.code !== 'BracketRight') return;
      e.preventDefault();
      const idx = TAB_ORDER.indexOf(tab);
      const delta = e.code === 'BracketRight' ? 1 : -1;
      changeTab(TAB_ORDER[(idx + delta + TAB_ORDER.length) % TAB_ORDER.length]);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [tab]);

  const library = state.library;

  useEffect(() => {
    if (!state.library?.watchlist && !state.library?.isLoading) {
      onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    }
  }, []);

  const selectedProviderLibrary = librarySource === 'local' ? null : providerLibraries[librarySource];
  const watchlist = (selectedProviderLibrary?.watchlist ?? library.lastWrite?.watchlist ?? library.watchlist ?? []) as LibraryItem[];
  const watching = (selectedProviderLibrary?.watching ?? library.lastWrite?.continueWatching ?? library.continueWatching ?? []) as LibraryItem[];
  const rawCompleted = (selectedProviderLibrary?.completed ?? library.lastWrite?.completed ?? library.completed ?? []) as LibraryItem[];
  const rawDropped = (selectedProviderLibrary?.dropped ?? library.lastWrite?.dropped ?? library.dropped ?? []) as LibraryItem[];
  const posterPrefs = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);
  const prefs = useMemo(() => appPrefs(state), [state.settings?.values]);
  const accent = prefString(prefs, 'accentColorArgb', '#FFFFFF');

  const homeCategories: HomeCategory[] = state.home.categories ?? [];

  const {
    collections,
    viewAllFolder, setViewAllFolder,
    editingCollection, setEditingCollection,
    collectionsScrollRef,
    openFolder,
    saveCollections,
    handleSaveCollection,
    handleDeleteCollection,
    handleImportJson,
    handleExportAll,
  } = useLibraryCollections({ activeProfile, onProfileUpdated, homeCategories });

  const [viewPlan, setViewPlan] = useState<{
    completed: LibraryItem[];
    dropped: LibraryItem[];
    smartLists: { airing: LibraryItem[]; rated: LibraryItem[]; history: LibraryItem[] };
    tabItems: LibraryItem[];
    items: LibraryItem[];
  }>({ completed: [], dropped: [], smartLists: { airing: [], rated: [], history: [] }, tabItems: [], items: [] });
  useEffect(() => {
    let active = true;
    void coreInvoke<typeof viewPlan>('libraryViewPlan', JSON.stringify({
      watchlist, watching, completed: rawCompleted, dropped: rawDropped,
      progress: library.lastWrite?.progress ?? {}, tab, query, sortBy,
    })).then((plan) => { if (active && plan) setViewPlan(plan); });
    return () => { active = false; };
  }, [watchlist, watching, rawCompleted, rawDropped, library.lastWrite?.progress, tab, query, sortBy]);
  const completed = viewPlan.completed;
  const dropped = viewPlan.dropped;
  const smartLists = viewPlan.smartLists;
  const items = viewPlan.tabItems;
  const sorted = viewPlan.items;

  const {
    bulkMode, setBulkMode,
    selectedIds, setSelectedIds,
    confirmBulkRemove, setConfirmBulkRemove,
    canRemoveFromCurrentList,
    toggleSelected, clearSelection,
    markSelectedWatched, moveSelectedToStatus, removeSelectedFromCurrentList,
  } = useLibraryBulkSelection({ items, tab, completed, dropped, onDispatch });

  const subtitle = tab === 'watchlist' ? t('auto.movies_and_shows_you_saved_to_watch_later')
    : tab === 'watching' ? t('library.subtitle_watching')
    : tab === 'completed' ? t('library.subtitle_completed')
    : tab === 'dropped' ? t('library.subtitle_dropped')
    : tab === 'airing' ? t('library.subtitle_airing')
    : tab === 'rated' ? t('library.subtitle_rated')
    : tab === 'history' ? t('library.subtitle_history')
    : t('library.subtitle_collections');
  const libraryTitle = librarySource === 'local'
    ? t('auto.my_library_a6c93797')
    : t(`library.source_${librarySource}`);

  if (viewAllFolder) {
    return (
      <CategoryGridScreen
        title={viewAllFolder.title}
        items={viewAllFolder.items}
        groups={viewAllFolder.groups}
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
      <div style={{ position: 'relative', height: '100%', paddingLeft: `${NAV_RAIL_WIDTH}rem`, background: '#040508', boxSizing: 'border-box' }}>
        <CollectionEditorScreen
          accent={accent}
          initial={initial}
          allCollections={collections}
          catalogOptions={homeCategories}
          onDismiss={() => setEditingCollection(null)}
          onSave={(c) => void handleSaveCollection(c)}
          onImportJson={(json) => void handleImportJson(json)}
          onExportAll={() => void handleExportAll()}
        />
      </div>
    );
  }

  return (
    <div style={styles.screen}>
      <div style={styles.header}>
        <CircleBtn onClick={onBack} size={48}>
          <ArrowLeft size={24} color="#fff" />
        </CircleBtn>
        <div>
          <p style={styles.title}>{libraryTitle}</p>
          <p style={styles.subtitle}>{subtitle}</p>
        </div>
      </div>

      <div style={styles.tabRow}>
        <TabChip active={tab === 'watchlist'} onClick={() => changeTab('watchlist')}>
          {t('library.plan_to_watch')}{watchlist.length > 0 ? ` (${watchlist.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'watching'} onClick={() => changeTab('watching')}>
          {t('library.watching')}{watching.length > 0 ? ` (${watching.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'completed'} onClick={() => changeTab('completed')}>
          {t('library.completed')}{completed.length > 0 ? ` (${completed.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'dropped'} onClick={() => changeTab('dropped')}>
          {t('library.dropped')}{dropped.length > 0 ? ` (${dropped.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'collections'} onClick={() => changeTab('collections')}>
          {t('library.collections')}{collections.length > 0 ? ` (${collections.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'airing'} onClick={() => changeTab('airing')}>
          {t('library.smart_airing')}{smartLists.airing.length > 0 ? ` (${smartLists.airing.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'rated'} onClick={() => changeTab('rated')}>
          {t('library.smart_rated')}{smartLists.rated.length > 0 ? ` (${smartLists.rated.length})` : ''}
        </TabChip>
        <TabChip active={tab === 'history'} onClick={() => changeTab('history')}>
          {t('library.history')}{smartLists.history.length > 0 ? ` (${smartLists.history.length})` : ''}
        </TabChip>
        {tab !== 'collections' && (
          <div style={styles.controls}>
            <div style={styles.searchWrap}>
              <Search size={15} style={{ color: 'rgba(255,255,255,0.35)', flexShrink: 0 }} />
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t('library.filter_placeholder')}
                style={styles.searchInput}
              />
            </div>
            <FilterDropdown
              value={sortBy === 'recent' ? t('library.sort_recent') : sortBy === 'title' ? t('library.sort_title') : t('library.sort_rating')}
              options={[
                { value: 'recent', label: t('library.sort_recent') },
                { value: 'title', label: t('library.sort_title') },
                { value: 'rating', label: t('library.sort_rating') },
              ]}
              onSelect={(v) => changeSort(v as 'recent' | 'title' | 'rating')}
            />
            <FilterDropdown
              value={t(`library.source_${librarySource}`)}
              options={[
                { value: 'local', label: t('library.source_local') },
                ...(providerLibraries.trakt ? [{ value: 'trakt', label: t('library.source_trakt') }] : []),
                ...(providerLibraries.simkl ? [{ value: 'simkl', label: t('library.source_simkl') }] : []),
                ...(providerLibraries.anilist ? [{ value: 'anilist', label: t('library.source_anilist') }] : []),
                ...(providerLibraries.nuvio ? [{ value: 'nuvio', label: t('library.source_nuvio') }] : []),
                ...(providerLibraries.stremio ? [{ value: 'stremio', label: t('library.source_stremio') }] : []),
              ]}
              onSelect={(v) => changeLibrarySource(v as LibrarySource)}
            />
            <button
              style={{ ...styles.bulkToggle, background: bulkMode ? '#FFFFFF' : 'rgba(255,255,255,0.05)', color: bulkMode ? '#000' : '#fff' }}
              onClick={() => {
                setBulkMode((v) => !v);
                clearSelection();
              }}
            >
              {bulkMode ? <CheckSquare2 size={15} /> : <Square size={15} />}
              <span>{t('library.bulk_select')}</span>
            </button>
          </div>
        )}
      </div>

      {bulkMode && tab !== 'collections' && (
        <div style={styles.bulkBar}>
          <button style={styles.bulkGhostBtn} onClick={() => {
            if (selectedIds.size === sorted.length) clearSelection();
            else setSelectedIds(new Set(sorted.map((item) => item.id)));
          }}>
            {selectedIds.size === sorted.length ? t('library.clear_selection') : t('library.select_all')}
          </button>
          <span style={styles.bulkCount}>{t('library.selected_count', selectedIds.size)}</span>
          <div style={{ flex: 1 }} />
          <button style={styles.bulkBtn} disabled={selectedIds.size === 0} onClick={() => markSelectedWatched(true)}>{t('detail.mark_watched')}</button>
          <button style={styles.bulkBtn} disabled={selectedIds.size === 0} onClick={() => markSelectedWatched(false)}>{t('detail.mark_unwatched')}</button>
          <button style={styles.bulkBtn} disabled={selectedIds.size === 0} onClick={() => moveSelectedToStatus('completed')}>{t('library.mark_completed')}</button>
          <button style={styles.bulkBtn} disabled={selectedIds.size === 0} onClick={() => moveSelectedToStatus('dropped')}>{t('library.mark_dropped')}</button>
          {canRemoveFromCurrentList && (
            <button style={styles.bulkDangerBtn} disabled={selectedIds.size === 0} onClick={() => setConfirmBulkRemove(true)}>{t('common.remove')}</button>
          )}
          <button style={styles.bulkIconBtn} onClick={() => { setBulkMode(false); clearSelection(); }} title={t('common.close')}><X size={17} /></button>
        </div>
      )}

      {library.lastWriteError && (
        <div style={styles.errorBanner}>
          <div style={{ minWidth: 0 }}>
            <p style={styles.errorTitle}>{t('common.error')}</p>
            <p style={styles.errorText}>{library.lastWriteError}</p>
          </div>
          <button style={styles.errorBtn} onClick={() => onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }))}>
            {t('common.retry')}
          </button>
        </div>
      )}

      <div style={{ height: '0.5rem' }} />

      {tab === 'collections' ? (
        <div ref={collectionsScrollRef} style={styles.collectionsScroll}>
          <CollectionsTab
            collections={collections}
            accent={accent}
            onFolderClick={(folder, folderTitle) => {
              void openFolder(folder, folderTitle);
            }}
            onEditCollection={(col) => setEditingCollection(col)}
            onDeleteCollection={(id) => void handleDeleteCollection(id)}
            onNewCollection={() => setEditingCollection('new')}
            onShowAllOnHome={() => void saveCollections(collections.map((c) => ({ ...c, showOnHome: true })))}
          />
        </div>
      ) : items.length === 0 ? (
        <div style={styles.empty}>
          <p style={styles.emptyTitle}>
            {tab === 'watchlist' ? t('library.your_list_empty')
              : tab === 'watching' ? t('library.nothing_to_continue')
              : tab === 'completed' ? t('library.nothing_completed')
              : tab === 'dropped' ? t('library.nothing_dropped')
              : tab === 'history' ? t('library.history_empty')
              : t('library.smart_empty')}
          </p>
          <p style={styles.emptyHint}>
            {tab === 'watchlist' ? t('library.add_titles_hint')
              : tab === 'watching' ? t('library.start_watching_hint')
              : tab === 'completed' ? t('library.completed_hint')
              : tab === 'dropped' ? t('library.dropped_hint')
              : tab === 'history' ? t('library.history_empty_hint')
              : t('library.smart_empty_hint')}
          </p>
        </div>
      ) : sorted.length === 0 ? (
        <div style={styles.empty}>
          <p style={styles.emptyTitle}>{t('library.no_matches')}</p>
        </div>
      ) : tab === 'history' ? (
        <HistoryTimeline items={sorted} onNavigateDetail={onNavigateDetail} />
      ) : (
        <VirtualizedPosterGrid
          items={sorted as unknown as Meta[]}
          selectedId={null}
          selectedIds={bulkMode ? selectedIds : undefined}
          posterPrefs={posterPrefs}
          onHover={() => false}
          onClick={bulkMode ? (item) => toggleSelected(item.id) : onNavigateDetail}
          onScrollActivity={() => {}}
        />
      )}

      {confirmBulkRemove && (
        <ConfirmDialog
          title={t('library.bulk_remove_confirm_title', selectedIds.size)}
          body={t('library.bulk_remove_confirm_body')}
          confirmLabel={t('common.remove')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmBulkRemove(false)}
          onConfirm={() => { setConfirmBulkRemove(false); removeSelectedFromCurrentList(); }}
        />
      )}
    </div>
  );
}, (prev, next) =>
  prev.state.library === next.state.library &&
  prev.state.settings === next.state.settings &&
  prev.state.home === next.state.home &&
  prev.activeProfile === next.activeProfile &&
  prev.onDispatch === next.onDispatch &&
  prev.onNavigateDetail === next.onNavigateDetail &&
  prev.onBack === next.onBack &&
  prev.onProfileUpdated === next.onProfileUpdated,
);
