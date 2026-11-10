use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ScrollKey {
    Screen(Route),
    Row(Route, usize),
    Chips,
    Strip(&'static str),
}

impl ScrollKey {
    pub(super) fn axis(self) -> usize {
        match self {
            Self::Screen(_) => 1,
            _ => 0,
        }
    }

    pub(super) fn id(self) -> fluxa_ui::ScrollId {
        match self {
            Self::Screen(_) => fluxa_ui::ScrollId::Screen,
            Self::Row(_, row) => fluxa_ui::ScrollId::Row(row),
            Self::Chips => fluxa_ui::ScrollId::Chips,
            Self::Strip(name) => fluxa_ui::ScrollId::Strip(name),
        }
    }
}

#[derive(Default)]
pub(super) struct ScrollStore {
    offsets: HashMap<ScrollKey, f32>,
}

impl ScrollStore {
    pub(super) fn reset(&mut self, key: ScrollKey) {
        self.offsets.remove(&key);
    }

    fn get(&self, key: ScrollKey) -> f32 {
        self.offsets.get(&key).copied().unwrap_or(0.0)
    }

    fn rows(&self, route: Route) -> Vec<f32> {
        let len = self
            .offsets
            .keys()
            .filter_map(|key| match key {
                ScrollKey::Row(owner, row) if *owner == route => Some(row + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        (0..len)
            .map(|row| self.get(ScrollKey::Row(route, row)))
            .collect()
    }
}

impl RendererState {
    pub(super) fn scroll_get(&self, key: ScrollKey) -> f32 {
        self.scroll.get(key)
    }

    pub(super) fn scroll_set(&mut self, key: ScrollKey, value: f32) {
        self.scroll.offsets.insert(key, value);
        self.scroll_mirror(key, value);
    }

    pub(super) fn scroll_reset(&mut self, key: ScrollKey) {
        self.scroll.offsets.remove(&key);
        self.scroll_mirror(key, 0.0);
    }

    pub(super) fn scroll_reset_rows(&mut self, route: Route) {
        let keys: Vec<_> = self
            .scroll
            .offsets
            .keys()
            .copied()
            .filter(|key| matches!(key, ScrollKey::Row(owner, _) if *owner == route))
            .collect();
        for key in keys {
            self.scroll_reset(key);
        }
    }

    pub(super) fn scroll_reset_screens(&mut self) {
        let keys: Vec<_> = self
            .scroll
            .offsets
            .keys()
            .copied()
            .filter(|key| matches!(key, ScrollKey::Screen(_)))
            .collect();
        for key in keys {
            self.scroll_reset(key);
        }
    }

    pub(super) fn scroll_sync_models(&mut self) {
        self.home.scroll_offset = self.scroll.get(ScrollKey::Screen(Route::Home));
        self.home.row_scroll_offsets = self.scroll.rows(Route::Home);
        self.folder.row_scroll_offsets = self.scroll.rows(Route::Folder);
        for row in 0..self.detail.row_scroll_offsets.len() {
            self.detail.row_scroll_offsets[row] =
                self.scroll.get(ScrollKey::Row(Route::Detail, row));
        }
        let chips = self.scroll.get(ScrollKey::Chips);
        if let Some(player) = self.player.as_mut() {
            player.source_chips_offset = chips;
        }
    }

    pub(super) fn scroll_push_strips(&self) {
        let Some(gpu) = self.gpu.as_ref() else {
            return;
        };
        for (key, value) in &self.scroll.offsets {
            if let ScrollKey::Strip(name) = key {
                fluxa_ui::set_strip_offset(fluxa_ui::gfx::Context::from_backend(&gpu.egui_context), name, *value);
            }
        }
    }

    fn scroll_mirror(&mut self, key: ScrollKey, value: f32) {
        match key {
            ScrollKey::Screen(Route::Home) => self.home.scroll_offset = value,
            ScrollKey::Row(Route::Home, row) => {
                mirror_row(&mut self.home.row_scroll_offsets, row, value)
            }
            ScrollKey::Row(Route::Folder, row) => {
                mirror_row(&mut self.folder.row_scroll_offsets, row, value)
            }
            ScrollKey::Row(Route::Detail, row) => {
                if let Some(offset) = self.detail.row_scroll_offsets.get_mut(row) {
                    *offset = value;
                }
            }
            ScrollKey::Chips => {
                if let Some(player) = self.player.as_mut() {
                    player.source_chips_offset = value;
                }
            }
            _ => {}
        }
    }
}

fn mirror_row(offsets: &mut Vec<f32>, row: usize, value: f32) {
    if offsets.len() <= row {
        offsets.resize(row + 1, 0.0);
    }
    offsets[row] = value;
}
