use super::{BadgeKind, BadgeLook};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    Classic,
    #[default]
    Vivid,
    Soft,
    Neon,
    Gold,
    Sunset,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "classic" => Self::Classic,
            "soft" => Self::Soft,
            "neon" => Self::Neon,
            "gold" => Self::Gold,
            "sunset" => Self::Sunset,
            _ => Self::Vivid,
        }
    }

    pub fn look(self, kind: BadgeKind) -> BadgeLook {
        let index = match kind {
            BadgeKind::Resolution => 0,
            BadgeKind::Source => 1,
            BadgeKind::Hdr => 2,
            BadgeKind::Codec => 3,
            BadgeKind::Audio => 4,
            BadgeKind::Size => 5,
            BadgeKind::Seeders | BadgeKind::Custom => 6,
        };
        let filled = |fill: u32| BadgeLook {
            fill: Some(fill),
            ..BadgeLook::default()
        };
        let outline = |color: u32| BadgeLook {
            fill: Some(color),
            outline: true,
            ..BadgeLook::default()
        };
        match self {
            Self::Classic => BadgeLook::default(),
            Self::Vivid => filled(
                [
                    0x2F6FEDFF, 0xE8742AFF, 0xC9A227FF, 0x7B4FD6FF, 0x1F9D6BFF, 0x3A3F4AFF,
                    0x14919BFF,
                ][index],
            ),
            Self::Soft => BadgeLook {
                text: Some(0x1A1A1AFF),
                ..filled(
                    [
                        0xBBD3FFFF, 0xFFD3B0FF, 0xFFEBA6FF, 0xDCC8FFFF, 0xBDECD3FF, 0xDADDE3FF,
                        0xB5E9EEFF,
                    ][index],
                )
            },
            Self::Neon => outline(
                [
                    0x4DA3FFFF, 0xFF8A3DFF, 0xFFD400FF, 0xB57BFFFF, 0x2DFFA0FF, 0xC8CCD4FF,
                    0x3DF2FFFF,
                ][index],
            ),
            Self::Gold if index == 0 => BadgeLook {
                text: Some(0x111111FF),
                ..filled(0xF2C14EFF)
            },
            Self::Gold => BadgeLook {
                border: Some(0xF2C14E99),
                ..outline(0xF2C14EFF)
            },
            Self::Sunset => filled(
                [
                    0xE5484DFF, 0xF76B15FF, 0xF5A524FF, 0xD6409FFF, 0xB4415BFF, 0x5B3A4AFF,
                    0xE07A5FFF,
                ][index],
            ),
        }
    }
}
