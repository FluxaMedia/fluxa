use super::*;

pub(crate) fn dialog<R>(
    context: &crate::gfx::Context,
    id: &str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    dialog_with(context, id, &[], add)
}

pub(crate) fn dialog_with<R>(
    context: &crate::gfx::Context,
    id: &str,
    modifiers: &[&'static str],
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let screen = context.content_rect();
    context
        .layer_painter(crate::gfx::LayerId::new(
            crate::gfx::Order::Middle,
            Id::new((id, "scrim")),
        ))
        .rect_filled(screen, 0.0, shade(Shade::Scrim));
    let style = crate::stylesheet::resolve_with(
        context,
        "dialog",
        Id::new(("dialog-rest", id)),
        crate::stylesheet::State {
            enabled: true,
            ..Default::default()
        },
        None,
        modifiers,
    );
    let margin = Space::S24.px(&crate::metrics::active());
    let width = (screen.width() - margin * 2.0).min(style.max_width.unwrap_or(f32::MAX));
    crate::gfx::Area::new(Id::new(id))
        .order(crate::gfx::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            let mut dialog = view().class("dialog").max_width(width);
            for modifier in modifiers {
                dialog = dialog.modifier(modifier);
            }
            dialog.show(ui, add).inner
        })
        .inner
}
