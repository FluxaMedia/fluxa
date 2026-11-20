use super::*;

pub(super) struct PrefetchPlan {
    pub(super) tree: Arc<Vec<Resolved>>,
    pub(super) sheet: Arc<crate::stylesheet::Sheet>,
    pub(super) metrics: Arc<UiMetrics>,
    pub(super) view: Vec2,
    pub(super) requests: Vec<(String, [u32; 2])>,
    pub(super) logos: Vec<(String, [u32; 2])>,
    pub(super) issued: Option<std::time::Instant>,
}

pub(super) const PREFETCH_REISSUE: std::time::Duration = std::time::Duration::from_millis(250);

pub(super) const PREFETCH_PLANS: usize = 6;

thread_local! {
    static PREFETCH: RefCell<Vec<PrefetchPlan>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn prefetch<A: HomeAssets>(env: &Env<'_, A>, tree: &Arc<Vec<Resolved>>) {
    let sheet = crate::stylesheet::sheet();
    let metrics = crate::metrics::active();
    PREFETCH.with(|plans| {
        let mut plans = plans.borrow_mut();
        let at = plans.iter().position(|plan| {
            Arc::ptr_eq(&plan.tree, tree)
                && Arc::ptr_eq(&plan.sheet, &sheet)
                && Arc::ptr_eq(&plan.metrics, &metrics)
                && plan.view == env.view.get()
        });
        let at = at.unwrap_or_else(|| {
            let mut requests = Vec::new();
            let mut logos = Vec::new();
            plan_prefetch(env, tree, None, None, &mut requests, &mut logos);
            if plans.len() >= PREFETCH_PLANS {
                plans.remove(0);
            }
            plans.push(PrefetchPlan {
                tree: tree.clone(),
                sheet,
                metrics,
                view: env.view.get(),
                requests,
                logos,
                issued: None,
            });
            plans.len() - 1
        });
        if plans[at]
            .issued
            .is_some_and(|issued| issued.elapsed() < PREFETCH_REISSUE)
        {
            return;
        }
        plans[at].issued = Some(std::time::Instant::now());
        let mut assets = env.assets.borrow_mut();
        for (url, target) in &plans[at].requests {
            assets.prefetch_for(Some(url), *target, ArtworkPriority::Prefetch);
        }
        for (id, target) in &plans[at].logos {
            let Some(card) = crate::card_registry::card(id) else {
                continue;
            };
            crate::card_logos::want(&card);
            if let Some(url) = crate::card_logos::logo_of(&card) {
                assets.prefetch_for(Some(&url), *target, ArtworkPriority::Prefetch);
            }
        }
    });
}

pub(super) fn plan_prefetch<A: HomeAssets>(
    env: &Env<'_, A>,
    nodes: &[Resolved],
    hint: Option<f32>,
    expand: Option<f32>,
    into: &mut Vec<(String, [u32; 2])>,
    logos: &mut Vec<(String, [u32; 2])>,
) {
    for node in nodes {
        let width = card_width(node, env).or(hint);
        let expand = if prop(node, "expand").is_empty() || !prop(node, "expandOff").is_empty() {
            expand
        } else {
            width
        };
        let expanded_only =
            node.props.get("expandedOnly").is_some_and(truthy) || !prop(node, "logoOf").is_empty();
        if expanded_only && matches!(node.kind.as_str(), "image" | "video") {
            let Some(base) = expand else {
                continue;
            };
            let logo = prop(node, "logoOf").into_owned();
            if !logo.is_empty() {
                let share = prop(node, "share").parse::<f32>().unwrap_or(1.0);
                let size = Vec2::new(
                    node.width
                        .unwrap_or_else(|| expanded_width(base, 1.0, env.metrics) * share),
                    node.height.unwrap_or(base * 1.5),
                );
                logos.push((logo, crate::artwork_target_size(size, 1.0)));
            } else if node.kind == "image"
                && let Some(url) = node.src.clone().filter(|url| !url.is_empty())
            {
                let size = Vec2::new(
                    expanded_width(base, 1.0, env.metrics),
                    node.height.unwrap_or(base * 1.5),
                );
                into.push((url, crate::artwork_target_size(size, 1.0)));
            }
            continue;
        }
        if node.kind == "image" {
            let style = class_parts(node, None)
                .map(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers));
            let size = match (width, style.and_then(|style| style.aspect)) {
                (Some(width), Some(ratio)) => Vec2::new(width, width / ratio),
                (Some(width), None) if node.fill => Vec2::new(width, width * 0.56),
                _ => Vec2::new(env.view.get().x, env.view.get().y * 0.62),
            };
            if let Some(url) = node.src.clone().filter(|url| !url.is_empty()) {
                into.push((url, crate::artwork_target_size(size, 1.0)));
            }
        }
        plan_prefetch(env, &node.children, width, expand, into, logos);
    }
}
