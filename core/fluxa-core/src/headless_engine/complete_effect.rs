use super::HeadlessEngine;
use super::contracts::EffectResultInput;
use super::{
    addons, auth, calendar, detail, discover, home, library, offline, player, plugins, search,
    settings, sync, trailer,
};
use crate::runtime::{EffectEnvelope, EffectKind};

impl HeadlessEngine {
    pub(super) fn complete_effect(&mut self, result: EffectResultInput) -> Vec<EffectEnvelope> {
        let Some(effect) = self.take_pending_effect(&result.effect_id) else {
            return vec![];
        };
        let generation = effect.generation;
        // A stale build mismatch still needs to clear the runtime registry entry. The FFI
        // boundary has no structured logging facility yet, so keep this non-fatal while making
        // the mismatch observable to hosts that collect stderr.
        let kind = effect.kind;
        let effect_type = kind.as_str();

        // No wildcard arm: adding an EffectKind variant without handling it here is a compile error.
        match kind {
            EffectKind::FetchMetaDetail
            | EffectKind::ReadPlaybackProgress
            | EffectKind::ReadDetailLocalState
            | EffectKind::FetchDetailSecondary
            | EffectKind::PrefetchDetailStreams
            | EffectKind::FetchDetailStreams
            | EffectKind::FetchMetaDetailLookup
            | EffectKind::FetchSeasonEpisodes => {
                detail::complete(self, effect_type, generation, &result)
            }

            EffectKind::LoadStreams
            | EffectKind::StartTorrentStream
            | EffectKind::EnqueueTraktScrobble
            | EffectKind::StopTorrent
            | EffectKind::FetchIntroSegments
            | EffectKind::ResolveIntroImdbId
            | EffectKind::FetchSubtitles
            | EffectKind::PrefetchNextEpisodeStreams => {
                player::complete(self, effect_type, generation, &result)
            }

            EffectKind::ReadHomeBootstrap
            | EffectKind::RefreshContinueWatching
            | EffectKind::PrepareDirectPlayback
            | EffectKind::FetchCatalogPage => {
                home::complete(self, effect_type, generation, &result)
            }

            EffectKind::ReadLibraryState
            | EffectKind::WriteLibraryCommand
            | EffectKind::WriteFeedback
            | EffectKind::ClearPlaybackProgress
            | EffectKind::WritePlaybackProgress
            | EffectKind::SyncWatchedState => {
                library::complete(self, effect_type, generation, &result)
            }

            EffectKind::FetchAddonManifest
            | EffectKind::RefreshInstalledAddons
            | EffectKind::FetchAddonResource => {
                addons::complete(self, effect_type, generation, &result)
            }

            EffectKind::RunSearch => search::complete(self, generation, &result),

            EffectKind::RunDiscover
            | EffectKind::ReadDiscoverCatalogFilters
            | EffectKind::FetchDiscoverPage => {
                discover::complete(self, effect_type, generation, &result)
            }

            EffectKind::ReadCalendarMonth => calendar::complete(self, generation, &result, &effect),

            EffectKind::EnqueueOfflineDownload => offline::complete(self, generation, &result),

            EffectKind::WriteSettings => settings::complete(self, generation, &result),

            EffectKind::RunExternalSync | EffectKind::SyncExternalIntegration => {
                sync::complete(self, effect_type, generation, &result)
            }

            EffectKind::RunAuthFlow
            | EffectKind::ExchangeAuthCode
            | EffectKind::RefreshAuthToken => {
                auth::complete(self, effect_type, generation, &result)
            }

            EffectKind::FetchYoutubeTrailerWatchConfig
            | EffectKind::FetchYoutubeTrailerPlayer
            | EffectKind::FetchYoutubeTrailerPlayerScript => {
                trailer::complete(self, effect_type, generation, &effect, &result)
            }

            EffectKind::FetchPluginManifest => plugins::complete(self, generation, &result),

            EffectKind::UpdateCalendarWidget
            | EffectKind::NotifyReleasedEpisodes
            | EffectKind::ReplaceExternalContinueWatching
            | EffectKind::ExecutePlugin => vec![],
        }
    }
}
