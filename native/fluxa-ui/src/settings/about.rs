use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

enum Notice {
    Key(&'static str),
    Verbatim(&'static str),
}

pub(super) struct Credit {
    logo: Option<&'static str>,
    name: &'static str,
    notice: Notice,
}

pub(super) fn credit_rows(language: &str) -> Vec<(String, String)> {
    CREDITS
        .iter()
        .map(|credit| (credit.name.to_owned(), notice(credit, language)))
        .collect()
}

const CREDITS: [Credit; 8] = [
    Credit {
        logo: None,
        name: "Fluxa",
        notice: Notice::Key("settings.about.credit.fluxa"),
    },
    Credit {
        logo: Some("tmdb"),
        name: "TMDB",
        notice: Notice::Verbatim(
            "This product uses the TMDB API but is not endorsed or certified by TMDB.",
        ),
    },
    Credit {
        logo: None,
        name: "mpv / FFmpeg / libass",
        notice: Notice::Key("settings.about.credit.playback"),
    },
    Credit {
        logo: None,
        name: "Phosphor Icons",
        notice: Notice::Key("settings.about.credit.phosphor"),
    },
    Credit {
        logo: None,
        name: "Archivo / Noto Sans",
        notice: Notice::Key("settings.about.credit.fonts"),
    },
    Credit {
        logo: None,
        name: "Anime4K",
        notice: Notice::Key("settings.about.credit.anime4k"),
    },
    Credit {
        logo: None,
        name: "epaint",
        notice: Notice::Key("settings.about.credit.epaint"),
    },
    Credit {
        logo: None,
        name: "librqbit-dualstack-sockets",
        notice: Notice::Key("settings.about.credit.librqbit"),
    },
];

fn notice(credit: &Credit, language: &str) -> String {
    match credit.notice {
        Notice::Key(key) => localized(key, language),
        Notice::Verbatim(text) => text.to_string(),
    }
}

pub fn settings_link_url(node: u64) -> Option<&'static str> {
    ABOUT_LINKS
        .iter()
        .find(|link| link.node == node)
        .map(|link| link.url)
        .filter(|url| !url.is_empty())
}

pub(super) struct AboutLink {
    pub(super) node: u64,
    pub(super) logo: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    url: &'static str,
}

pub(super) const ABOUT_LINKS: [AboutLink; 3] = [
    AboutLink {
        node: NODE_SETTINGS_LINK_GITHUB,
        logo: "github",
        title: "settings.about.github",
        subtitle: "settings.about.github_value",
        url: "https://github.com/FluxaMedia/fluxa",
    },
    AboutLink {
        node: NODE_SETTINGS_LINK_DISCORD,
        logo: "discord",
        title: "settings.about.discord",
        subtitle: "settings.about.discord_value",
        url: "https://discord.gg/wan9FeDEfe",
    },
    AboutLink {
        node: NODE_SETTINGS_REPORT_PROBLEM,
        logo: "report",
        title: "settings.about.report",
        subtitle: "settings.about.report_value",
        url: "",
    },
];

pub(super) fn about_update(update: &UpdateView, language: &str) -> (String, Option<&'static str>) {
    match &update.state {
        _ if !update.supported => (localized("settings.updates.unsupported", language), None),
        UpdateState::Idle => (
            localized("settings.updates.idle", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Checking => (localized("settings.updates.checking", language), None),
        UpdateState::UpToDate => (
            localized("settings.updates.up_to_date", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Available(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.available", language)
            ),
            Some("settings.updates.install"),
        ),
        UpdateState::Downloading(percent) => (
            format!(
                "{} {percent}%",
                localized("settings.updates.downloading", language)
            ),
            None,
        ),
        UpdateState::Ready(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.ready", language)
            ),
            Some("settings.updates.restart"),
        ),
        UpdateState::Failed(reason) => (
            format!(
                "{}: {reason}",
                localized("settings.updates.failed", language)
            ),
            Some("settings.updates.check"),
        ),
    }
}
