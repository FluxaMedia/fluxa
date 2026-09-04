export interface Meta {
  id: string;
  type: string;
  name: string;
  poster?: string;
  background?: string;
  logo?: string;
  network?: string;
  description?: string;
  tagline?: string;
  status?: string;
  awards?: string;
  year?: number;
  imdbRating?: number;
  certification?: string;
  genres?: string[];
  keywords?: string[];
  runtime?: string;
  releaseInfo?: string;
  focusGifUrl?: string;
  focusGifEnabled?: boolean;
  coverEmoji?: string;
  hideTitle?: boolean;
  cast?: CastMember[];
  director?: string[];
  createdBy?: string[];
  collection?: MetaCollection;
  originalLanguage?: string;
  productionCountries?: string[];
  alternativeTitles?: string[];
  nextEpisodeToAir?: EpisodeToAir;
  lastEpisodeToAir?: EpisodeToAir;
  watchProviders?: WatchProviders;
  trailers?: Trailer[];
  links?: MetaLink[];
  videos?: Video[];
  sourceAddonTransportUrl?: string;
  sourceAddonCatalogType?: string;
}
export interface MetaCollection {
  name: string;
  poster?: string;
  background?: string;
}

export interface EpisodeToAir {
  season: number;
  episode: number;
  airDate?: string;
  name?: string;
}

export interface WatchProvider {
  name: string;
  logo?: string;
}

export interface WatchProviders {
  region: string;
  link?: string;
  flatrate?: WatchProvider[];
  rent?: WatchProvider[];
  buy?: WatchProvider[];
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
  extra?: Record<string, unknown>;
}

export interface StreamBadge {
  name: string;
  imageUrl?: string;
  tagColor?: string;
  tagStyle?: string;
  textColor?: string;
  borderColor?: string;
}

export interface StreamBadgeFilter {
  id?: string;
  groupId?: string;
  name: string;
  pattern: string;
  imageUrl?: string;
  isEnabled?: boolean;
  tagColor?: string;
  tagStyle?: string;
  textColor?: string;
  borderColor?: string;
}

export interface StreamBadgeGroup {
  id?: string;
  name?: string;
  color?: string;
  isExpanded?: boolean;
}

export interface StreamBadgeImport {
  sourceUrl: string;
  filters: StreamBadgeFilter[];
  groups: StreamBadgeGroup[];
  isActive?: boolean;
}

export interface StreamBadgeRules {
  imports: StreamBadgeImport[];
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
  focusGlowEnabled?: boolean;
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
  resumeProgressPercent?: number;
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
  traktUsername?: string;
  anilistAccessToken?: string;
  anilistRefreshToken?: string;
  anilistTokenExpiresAt?: number;
  anilistUsername?: string;
  simklAccessToken?: string;
  simklRefreshToken?: string;
  simklTokenExpiresAt?: number;
  simklUsername?: string;
  stremioAuthKey?: string;
  stremioEmail?: string;
  nuvioAccessToken?: string;
  nuvioRefreshToken?: string;
  nuvioTokenExpiresAt?: number;
  nuvioUserId?: string;
  pinHash?: string;
  fluxaProfileId?: string;
  nuvioEmail?: string;
  nuvioProfileIndex?: number;
  nuvioPinEnabled?: boolean;
  nuvioPinLockedUntil?: string | null;
  nuvioProfileUpdatedAt?: string;
  usesPrimaryAddons?: boolean;
  usesPrimaryPlugins?: boolean;
}
