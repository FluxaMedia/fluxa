import type { EffectType } from './contracts';

export interface Meta {
  id: string;
  type: string;
  name: string;
  poster?: string;
  background?: string;
  logo?: string;
  description?: string;
  awards?: string;
  year?: number;
  imdbRating?: number;
  genres?: string[];
  runtime?: string;
  releaseInfo?: string;
  focusGifUrl?: string;
  cast?: CastMember[];
  director?: string[];
  trailers?: Trailer[];
  links?: MetaLink[];
  videos?: Video[];
  sourceAddonTransportUrl?: string;
  sourceAddonCatalogType?: string;
}

export interface CastMember {
  name: string;
  character?: string | null;
  profilePath?: string | null;
  profile_path?: string | null;
  photo?: string | null;
  profile?: string | null;
  image?: string | null;
  img?: string | null;
  firstName?: string | null;
  first_name?: string | null;
  lastName?: string | null;
  last_name?: string | null;
  surname?: string | null;
  familyName?: string | null;
  family_name?: string | null;
}

export interface Trailer {
  url: string;
  title?: string;
  type?: string;
}

export interface MetaLink {
  name: string;
  category: string;
  url: string;
}

export interface Video {
  id: string;
  title?: string;
  name?: string;
  season?: number;
  episode?: number;
  number?: number;
  released?: string;
  thumbnail?: string;
  overview?: string;
}

export interface Stream {
  url?: string;
  externalUrl?: string;
  playerFrameUrl?: string;
  infoHash?: string;
  fileIdx?: number;
  name?: string;
  title?: string;
  description?: string;
  addonName?: string;
  subtitles?: SubtitleTrack[];
  behaviorHints?: BehaviorHints;
  playableUrl?: string;
  isTorrent?: boolean;
  sources?: string[];
}

export interface BehaviorHints {
  notWebReady?: boolean;
  bingeGroup?: string;
  requestHeaders?: Record<string, string>;
  proxyHeaders?: {
    request?: Record<string, string>;
  };
  videoHash?: string;
  videoSize?: number;
  filename?: string;
}

export interface SubtitleTrack {
  url: string;
  lang: string;
  label?: string;
}

export interface HomeCatalogSource {
  transportUrl: string;
  catalogId: string;
  type: string;
  genre?: string;
}

export interface HomeCategory {
  id: string;
  name: string;
  semanticName?: string;
  type: string;
  items: Meta[];
  addonName?: string;
  transportUrl?: string;
  catalogId?: string;
  addonGenre?: string;
  catalogSources?: HomeCatalogSource[];
  hasMore?: boolean;
}

export interface LibraryItem {
  id: string;
  name: string;
  type: string;
  poster?: string;
  background?: string;
  logo?: string;
  timeOffset?: number;
  duration?: number;
  lastVideoId?: string;
  lastStreamIndex?: number;
  lastStreamUrl?: string;
  lastStreamTitle?: string;
  lastStream?: Stream;
  lastEpisodeName?: string;
  lastEpisodeSeason?: number;
  lastEpisodeNumber?: number;
  lastEpisodeThumbnail?: string;
  lastAudioLanguage?: string;
  lastSubtitleLanguage?: string;
  continueWatchingBadge?: 'newEpisode' | 'upNext' | string;
  newEpisodeReleasedAt?: string;
  inWatchlist?: boolean;
  nextEpisodeAirDate?: string;
  nextEpisodeSeason?: number;
  nextEpisodeNumber?: number;
  nextEpisodeTitle?: string;
  nextEpisodePoster?: string;
  lastAirDateCheckedAt?: string;
  statusChangedAt?: string;
}

export interface AddonDescriptor {
  manifest: AddonManifest;
  transportUrl: string;
  id?: string;
  name?: string;
  version?: string;
  description?: string;
  logo?: string;
  background?: string;
  resources?: Array<string | AddonResourceSpec>;
  types?: string[];
  catalogs?: CatalogDef[];
  behaviorHints?: { configurable?: boolean; adult?: boolean };
}

export interface AddonManifest {
  id: string;
  name: string;
  description?: string | null;
  version?: string | null;
  resources?: Array<string | AddonResourceSpec>;
  types?: string[];
  catalogs?: CatalogDef[];
  idPrefixes?: string[] | null;
  logo?: string | null;
  background?: string | null;
  configurable?: boolean | null;
}

export interface AddonResourceSpec {
  name?: string;
  type?: string;
  types?: string[];
  idPrefixes?: string[];
  idPrefix?: string[];
}

export interface CatalogDef {
  type: string;
  id: string;
  name?: string;
  extra?: ExtraDef[];
  extraSupported?: string[];
}

export interface ExtraDef {
  name: string;
  isRequired?: boolean;
  options?: string[];
}

export interface CatalogSource {
  addonId?: string;
  catalogId: string;
  type: string;
  genre?: string;
}

export interface NuvioAddonCollectionSource {
  provider: 'addon';
  addonId: string;
  type: string;
  catalogId: string;
  genre?: string;
}

export interface NuvioRemoteCollectionSource {
  provider: 'trakt' | 'tmdb';
  title?: string;
  mediaType?: string;
  traktListId?: number;
  tmdbSourceType?: string;
  tmdbId?: number;
  sortBy?: string;
  sortHow?: string;
  filters?: Record<string, unknown>;
}

export type NuvioCollectionSource = NuvioAddonCollectionSource | NuvioRemoteCollectionSource;

export interface UserCollectionFolder {
  id: string;
  title: string;
  imageUrl?: string;
  shape?: string;
  catalogId?: string;
  catalogTitle?: string;
  genre?: string;
  hideTitle?: boolean;
  focusGifEnabled?: boolean;
  catalogSources?: CatalogSource[];
  sources?: NuvioCollectionSource[];
  coverEmoji?: string;
  coverImageUrl?: string;
  focusGifUrl?: string;
  titleLogoUrl?: string;
  heroBackdropUrl?: string;
  heroVideoUrl?: string;
}

export interface UserCollection {
  id: string;
  title: string;
  itemIds?: string[];
  imageUrl?: string;
  backdropImageUrl?: string;
  showOnHome?: boolean;
  folders?: UserCollectionFolder[];
  showAllTab?: boolean;
  viewMode?: string;
  pinToTop?: boolean;
  focusGlowEnabled?: boolean;
}

export interface UserProfile {
  id: string;
  name?: string;
  avatarUrl?: string;
  isAnonymous?: boolean;
  email?: string;
  libraryCollections?: UserCollection[];
  color?: string;
  localAddons?: string[];
  disabledLocalAddons?: string[];
  addonSettings?: {
    localAddons?: string[];
    disabledLocalAddons?: string[];
  };
  traktAccessToken?: string;
  traktRefreshToken?: string;
  traktTokenExpiresAt?: number;
  anilistAccessToken?: string;
  anilistRefreshToken?: string;
  anilistTokenExpiresAt?: number;
  simklAccessToken?: string;
  simklRefreshToken?: string;
  simklTokenExpiresAt?: number;
  stremioAuthKey?: string;
  stremioEmail?: string;
  nuvioAccessToken?: string;
  nuvioRefreshToken?: string;
  nuvioTokenExpiresAt?: number;
  nuvioUserId?: string;
  pinHash?: string;
  nuvioEmail?: string;
  nuvioProfileIndex?: number;
  usesPrimaryAddons?: boolean;
  usesPrimaryPlugins?: boolean;
}

export interface Effect {
  id: string;
  type: EffectType;
  generation: number;
  payload: Record<string, unknown>;
}

export interface EffectResult {
  effectId: string;
  status: 'ok' | 'err';
  value?: unknown;
  error?: unknown;
}

export interface DispatchResult {
  // Only the domains that changed since the previous dispatch are present —
  // see mergeAppState, which merges this onto existing state instead of replacing it.
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
  isLoading?: boolean;
  lastWrite?: {
    watchlist?: LibraryItem[];
    continueWatching?: LibraryItem[];
    dropped?: LibraryItem[];
    completed?: LibraryItem[];
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
