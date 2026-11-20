use super::*;

pub(super) static GRID_WINDOWS: Mutex<fluxa_skin::Windows> = Mutex::new(fluxa_skin::Windows::new());

pub(super) static STRIP_NAMES: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
pub(super) static FLOW_KEYS: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());

pub fn row_strip(index: usize) -> Option<&'static str> {
    let flow = FLOW_KEYS.lock().ok()?;
    let prefix = format!("{}.", flow.iter().find(|(row, _)| *row == index)?.1);
    let names = STRIP_NAMES.lock().ok()?;
    names
        .as_ref()?
        .iter()
        .find(|(key, _)| key.starts_with(&prefix))
        .map(|(_, name)| *name)
}

pub(super) fn strip_name(key: &str) -> &'static str {
    let mut names = STRIP_NAMES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let names = names.get_or_insert_with(HashMap::new);
    if let Some(name) = names.get(key) {
        return name;
    }
    let name: &'static str = Box::leak(format!("fluxa-skin-strip:{key}").into_boxed_str());
    names.insert(key.to_owned(), name);
    name
}

thread_local! {
    static FLOW_NEAR: RefCell<Positions> = RefCell::new(Positions::default());
}

pub(super) fn flow_owner(key: &str) -> Option<(&str, usize)> {
    let (each, index) = key.rsplit_once('.')?;
    let (owner, _) = each.rsplit_once('.')?;
    Some((owner, index.parse().ok()?))
}

pub(super) fn flow_report(ctx: &crate::gfx::Context, key: &str, rect: Rect, clip: Rect) {
    let id = Id::new(("skin-flow-height", key));
    if rect.height() > 0.0 {
        ctx.data_mut(|data| {
            data.insert_temp(id, rect.height());
            data.insert_temp(Id::new("skin-flow-estimate"), rect.height());
        });
    }
    let near =
        rect.bottom() > clip.top() - clip.height() && rect.top() < clip.bottom() + clip.height();
    if !near {
        return;
    }
    if let Some((owner, index)) = flow_owner(key) {
        if let Ok(mut keys) = FLOW_KEYS.lock() {
            match keys.iter_mut().find(|(row, _)| *row == index) {
                Some(entry) => entry.1 = key.to_owned(),
                None => keys.push((index, key.to_owned())),
            }
        }
        FLOW_NEAR.with(|near| {
            let mut near = near.borrow_mut();
            let range = near.entry(owner.to_owned()).or_insert((index, index));
            range.0 = range.0.min(index);
            range.1 = range.1.max(index);
        });
    }
}

pub(super) fn flow_publish(ctx: &crate::gfx::Context) {
    let near = FLOW_NEAR.with(|near| std::mem::take(&mut *near.borrow_mut()));
    let Ok(mut windows) = GRID_WINDOWS.lock() else {
        return;
    };
    for (owner, (first, last)) in near {
        let start = first.saturating_sub(1) / 2 * 2;
        let end = (last + 2).div_ceil(2) * 2;
        if windows.get(&owner) != Some(&(start, end)) {
            windows.insert(owner, (start, end));
            ctx.request_repaint();
        }
    }
}

pub(super) fn flow_far<'a>(ui: &Ui, node: &Resolved) -> kit::Item<'a> {
    let ctx = ui.ctx().clone();
    let height = ctx
        .data(|data| {
            data.get_temp::<f32>(Id::new(("skin-flow-height", prop(node, "windowed"))))
                .or_else(|| data.get_temp::<f32>(Id::new("skin-flow-estimate")))
        })
        .unwrap_or(240.0);
    let key = prop(node, "windowed").into_owned();
    kit::shared(Vec2::new(0.0, height), move |ui, rect| {
        let clip = ui.clip_rect();
        flow_report(&ctx, &key, rect, clip);
    })
}

pub fn grid_windows() -> fluxa_skin::Windows {
    GRID_WINDOWS
        .lock()
        .map(|windows| windows.clone())
        .unwrap_or_default()
}
