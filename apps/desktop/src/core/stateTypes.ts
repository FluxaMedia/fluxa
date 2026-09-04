import type { Effect } from './contracts';
import type {
  AddonDescriptor,
  HomeCategory,
  LibraryItem,
  Meta,
  Stream,
  Trailer,
  UserProfile,
  Video,
} from './models';

export interface DispatchResult {
  revision?: number;
  // Only changed domains are present; consumers merge this patch onto existing state.
  state: Partial<AppState>;
  effects: Effect[];
}

export interface HomePaging {
  categoryId?: string;
  isLoading?: boolean;
  items?: Meta[];
  error?: string | null;
}

export interface HomeState {
  isLoading?: boolean;
  isStale?: boolean;
  isDirectLoading?: boolean;
  categories?: HomeCategory[];
  continueWatching?: LibraryItem[];
  metadataFeeds?: unknown[];
  billboard?: Meta | null;
  error?: string | null;
  paging?: HomePaging;
}

export interface DetailState {
  isLoading?: boolean;
  isLoadingStreams?: boolean;
  meta?: Meta | null;
  streams?: Stream[];
  visibleStreams?: Stream[];
  availableAddons?: string[];
  failedAddons?: string[];
  selectedAddon?: string | null;
  trailers?: Trailer[];
  similarItems?: Meta[];
  omdbRatings?: { rottenTomatoes?: string; metascore?: string } | null;
  mdblistRatings?: Record<string, number> | null;
  fanartArtwork?: { hdLogo?: string; hdBackdrop?: string } | null;
  error?: string | null;
  id?: string;
  type?: string;
  seasonEpisodes?: Video[];
  selectedSeason?: number;
}

export interface SearchState {
  query?: string;
  isLoading?: boolean;
  results?: Meta[];
  categories?: HomeCategory[];
  grouping?: unknown;
  error?: string | null;
}

export interface PlayerState {
  currentVideoId?: string;
  currentStreamIndex?: number;
  currentStreams?: Stream[];
  currentUrl?: string;
  resolvedUrl?: string;
  isBuffering?: boolean;
  playerError?: string | null;
}

export interface LibraryStateSlice {
  watchlist?: LibraryItem[];
  continueWatching?: LibraryItem[];
  dropped?: LibraryItem[];
  completed?: LibraryItem[];
  favorites?: LibraryItem[];
  isLoading?: boolean;
  lastWrite?: {
    watchlist?: LibraryItem[];
    continueWatching?: LibraryItem[];
    dropped?: LibraryItem[];
    completed?: LibraryItem[];
    favorites?: LibraryItem[];
    progress?: Record<string, LibraryItem>;
  };
  lastWriteError?: string | null;
}

export interface DiscoverPaging {
  isLoading?: boolean;
  items?: Meta[];
  error?: string | null;
}

export interface DiscoverState {
  isLoading?: boolean;
  catalogsLoading?: boolean;
  results?: Meta[];
  filters?: DiscoverFilter[];
  catalogs?: DiscoverCatalog[];
  selectedCatalogKey?: string | null;
  error?: string | null;
  paging?: DiscoverPaging;
}

export interface DiscoverCatalog {
  key: string;
  label: string;
  type: string;
  extras?: DiscoverFilter[];
  transportUrl?: string;
  id?: string;
}

export interface DiscoverFilter {
  name: string;
  options?: string[];
  isRequired?: boolean;
}

export interface AddonsState {
  installed?: AddonDescriptor[];
  isLoading?: boolean;
  error?: string | null;
}

export interface PluginRepository {
  manifestUrl: string;
  name?: string;
  description?: string;
  version?: string;
  scraperCount?: number;
}

export interface PluginScraper {
  id: string;
  name: string;
  repositoryUrl: string;
  filename?: string;
  enabled: boolean;
  supportedTypes?: string[];
}

export interface PluginsState {
  repositories?: PluginRepository[];
  scrapers?: PluginScraper[];
  addingRepositoryUrl?: string | null;
  error?: { code?: string; message?: string } | string | null;
}

export interface SettingsState {
  values?: Record<string, unknown>;
}

export interface NavigationState {
  route: string;
  params?: Record<string, unknown> | null;
}

export interface AppState {
  navigation: NavigationState;
  home: HomeState;
  detail: DetailState;
  search: SearchState;
  player: PlayerState;
  library: LibraryStateSlice;
  discover: DiscoverState;
  addons: AddonsState;
  plugins?: PluginsState;
  settings: SettingsState;
  profile: { active?: UserProfile };
  pendingEffects: Effect[];
  auth?: unknown;
  sync?: unknown;
  calendar?: unknown;
  offline?: unknown;
  trailer?: { resolutions?: Record<string, unknown> };
}
