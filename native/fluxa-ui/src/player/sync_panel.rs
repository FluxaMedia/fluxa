use super::panel::SyncRole;
use super::*;

pub(super) fn draw(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    slide: f32,
) {
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let form_factor = chrome.viewport.form_factor;
    let text = super::tokens::PlayerTokens::for_form_factor(form_factor).list_text;
    let pad = text * 1.15;
    let mobile = form_factor == UiFormFactor::Mobile;
    let sheet = mobile;
    let available = chrome.rect.width() - chrome.margin() * 2.0;
    let width = (text * 30.35).min(if sheet {
        available.min(560.0)
    } else {
        available
    });

    let head_h = text * 3.0;
    let reading_h = text * 3.6;
    let auto_h = text * 3.3;
    let method_h = text * 3.8;
    let choice_h = text * 2.7;
    let gap = text * 1.1;
    let handle_h = if sheet { 12.0 } else { 0.0 };

    let reserved = match form_factor {
        UiFormFactor::Desktop => 120.0 + chrome.viewport.safe_bottom,
        _ if sheet => chrome.viewport.safe_bottom,
        _ => 32.0,
    };
    let available_height = if sheet {
        chrome.rect.height() * 0.88
    } else {
        chrome.rect.height()
    };
    let methods = panel
        .rows
        .iter()
        .filter(|row| row.sync == Some(SyncRole::Method))
        .count();
    let status = find_row(panel, SyncRole::Reading).and_then(|row| row.value.as_deref());
    let hint = find_row(panel, SyncRole::Auto).and_then(|row| row.detail.as_deref());
    let status_h = if status.is_some() { text * 2.6 } else { 0.0 };
    let hint_h = if hint.is_some() { text * 2.5 } else { 0.0 };
    let fixed = pad * 2.0
        + handle_h
        + head_h
        + reading_h
        + status_h
        + gap
        + auto_h
        + hint_h
        + gap
        + methods as f32 * method_h;
    let room = available_height - reserved - chrome.margin() - fixed;
    let shown_choices = ((room / choice_h).floor().max(1.0)) as usize;
    let choices = panel
        .rows
        .iter()
        .filter(|row| row.sync == Some(SyncRole::Choice))
        .count()
        .min(shown_choices);
    let height = fixed + choices as f32 * choice_h;

    let card = match form_factor {
        UiFormFactor::Desktop => Rect::from_min_size(
            Pos2::new(
                chrome.rect.right() - chrome.margin() - width,
                chrome.rect.bottom() - reserved - height,
            ),
            Vec2::new(width, height),
        ),
        _ if sheet => Rect::from_min_max(
            Pos2::new(
                chrome.rect.center().x - width * 0.5,
                chrome.rect.bottom() - height - reserved,
            ),
            Pos2::new(chrome.rect.center().x + width * 0.5, chrome.rect.bottom()),
        ),
        _ => Rect::from_center_size(chrome.rect.center(), Vec2::new(width, height)),
    };

    let veil = chrome.context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-panel-veil"),
    ));
    let outside = ui.interact(
        chrome.rect,
        Id::new("fluxa-player-panel-outside"),
        Sense::click(),
    );
    if outside.clicked()
        && !outside
            .interact_pointer_pos()
            .is_some_and(|pos| card.contains(pos))
    {
        layout.activated = Some(NODE_PLAYER_PANEL_CLOSE);
    }
    if sheet {
        veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    } else if form_factor != UiFormFactor::Desktop {
        veil.rect_filled(chrome.rect, 0.0, shade(Shade::Veil));
    }
    let layer = veil.layer_id();
    let first = chrome
        .context
        .graphics_mut(|graphics| graphics.entry(layer).next_idx());
    let radius = if sheet {
        egui::CornerRadius {
            nw: 24,
            ne: 24,
            sw: 0,
            se: 0,
        }
    } else {
        egui::CornerRadius::same(tokens.card_radius as u8)
    };
    veil.rect_filled(card, radius, tokens.surface);
    veil.rect_stroke(
        card,
        radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    if sheet {
        veil.rect_filled(
            Rect::from_center_size(
                Pos2::new(card.center().x, card.top() + 9.0),
                Vec2::new(36.0, 4.0),
            ),
            2.0,
            ink(tone::strong()),
        );
    }

    let icon = |name: &str, center: Pos2, side: f32, color: Color32| {
        if let Some(texture) = chrome.icons.get(name) {
            veil.image(
                texture,
                Rect::from_center_size(center, Vec2::splat(side)),
                full_uv(),
                color,
            );
        }
    };
    let find = |role: SyncRole| {
        panel
            .rows
            .iter()
            .enumerate()
            .find(|(_, row)| row.sync == Some(role))
    };
    let node_of = |index: usize| NODE_PLAYER_PANEL_ROW_BASE + index as u64;
    let left = card.left() + pad;
    let right = card.right() - pad;
    let mut y = card.top() + pad * 0.6 + handle_h;

    let back_side = text * 2.4;
    let head = Rect::from_min_size(
        Pos2::new(left - 6.0, y),
        Vec2::new(right - left + 12.0, head_h),
    );
    if let Some((index, row)) = find(SyncRole::Back) {
        let hit = Rect::from_center_size(
            Pos2::new(left + back_side * 0.5 - 4.0, head.center().y),
            Vec2::splat(back_side),
        );
        let node = node_of(index);
        let response = ui.interact(hit, Id::new(("fluxa-sync-back", node)), Sense::click());
        layout.focusable.push((node, hit));
        let on = chrome.focused == Some(node);
        if on || response.hovered() {
            veil.circle_filled(
                hit.center(),
                back_side * 0.5,
                if on {
                    text_primary()
                } else {
                    ink(tone::soft())
                },
            );
        }
        icon(
            "ArrowLeft",
            hit.center(),
            text * 1.35,
            if on { Color32::BLACK } else { ink(215) },
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
        veil.text(
            Pos2::new(hit.right() + text * 0.7, head.center().y),
            Align2::LEFT_CENTER,
            &row.label,
            crate::fonts::bold(text * 1.15),
            text_primary(),
        );
    }
    if let Some((index, row)) = find(SyncRole::Reset) {
        let font = FontId::proportional(text * 0.95);
        let size = crate::flex::text_size(&veil, &row.label, &font);
        let hit = Rect::from_center_size(
            Pos2::new(right - size.x * 0.5 - text * 0.6, head.center().y),
            Vec2::new(size.x + text * 1.2, text * 2.2),
        );
        let node = node_of(index);
        let response = ui.interact(hit, Id::new(("fluxa-sync-reset", node)), Sense::click());
        layout.focusable.push((node, hit));
        let on = chrome.focused == Some(node);
        if on || response.hovered() {
            veil.rect_filled(
                hit,
                hit.height() * 0.5,
                if on {
                    text_primary()
                } else {
                    ink(tone::soft())
                },
            );
        }
        veil.text(
            hit.center(),
            Align2::CENTER_CENTER,
            &row.label,
            font,
            if on {
                Color32::BLACK
            } else {
                ink(tone::muted())
            },
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    y += head_h;

    let reading = Rect::from_min_size(Pos2::new(left, y), Vec2::new(right - left, reading_h));
    if let Some((_, row)) = find(SyncRole::Reading) {
        let big = crate::fonts::bold(text * 2.85);
        let unit = crate::fonts::bold(text * 1.3);
        let (number, suffix) = row
            .label
            .strip_suffix('s')
            .map_or((row.label.as_str(), ""), |number| (number, "s"));
        let number_size = crate::flex::text_size(&veil, number, &big);
        veil.text(
            Pos2::new(reading.left(), reading.center().y),
            Align2::LEFT_CENTER,
            number,
            big,
            text_primary(),
        );
        veil.text(
            Pos2::new(
                reading.left() + number_size.x + 3.0,
                reading.center().y + text * 0.55,
            ),
            Align2::LEFT_CENTER,
            suffix,
            unit,
            ink(tone::muted()),
        );
        let stepper_w = text * 8.6;
        let pill = Rect::from_center_size(
            Pos2::new(reading.right() - stepper_w * 0.5, reading.center().y),
            Vec2::new(stepper_w, text * 2.6),
        );
        veil.rect_filled(pill, pill.height() * 0.5, ink(tone::soft()));
        veil.rect_stroke(
            pill,
            pill.height() * 0.5,
            egui::Stroke::new(stroke::hairline(), tokens.border),
            egui::StrokeKind::Inside,
        );
        veil.line_segment(
            [
                pill.center_top() + Vec2::new(0.0, 6.0),
                pill.center_bottom() - Vec2::new(0.0, 6.0),
            ],
            egui::Stroke::new(stroke::hairline(), tokens.border),
        );
        for (role, half) in [
            (
                SyncRole::Minus,
                Rect::from_min_max(pill.min, pill.center_bottom()),
            ),
            (
                SyncRole::Plus,
                Rect::from_min_max(pill.center_top(), pill.max),
            ),
        ] {
            let Some((index, step)) = find(role) else {
                continue;
            };
            let node = node_of(index);
            let response = ui.interact(half, Id::new(("fluxa-sync-step", node)), Sense::click());
            layout.focusable.push((node, half));
            let on = chrome.focused == Some(node);
            if on {
                veil.rect_filled(half.shrink(1.0), pill.height() * 0.5, text_primary());
            } else if response.is_pointer_button_down_on() {
                veil.rect_filled(half.shrink(1.0), pill.height() * 0.5, ink(tone::strong()));
            } else if response.hovered() {
                veil.rect_filled(half.shrink(1.0), pill.height() * 0.5, ink(tone::faint()));
            }
            veil.text(
                half.center(),
                Align2::CENTER_CENTER,
                &step.label,
                crate::fonts::bold(text * 0.93),
                if on { Color32::BLACK } else { text_primary() },
            );
            if response.clicked() {
                layout.activated = Some(node);
            }
        }
    }
    y += reading_h;
    if let Some(status) = status {
        let mut job = egui::text::LayoutJob::simple(
            status.to_owned(),
            FontId::proportional(text * 0.95),
            ink(tone::subtle()),
            right - left,
        );
        job.wrap.max_rows = 2;
        let galley = veil.layout_job(job);
        veil.galley(Pos2::new(left, y + text * 0.2), galley, ink(tone::subtle()));
        y += status_h;
    }
    y += gap;

    if let Some((index, row)) = find(SyncRole::Auto) {
        let rect = Rect::from_min_size(Pos2::new(left, y), Vec2::new(right - left, auto_h));
        let node = node_of(index);
        let response = ui.interact(rect, Id::new(("fluxa-sync-auto", node)), Sense::click());
        layout.focusable.push((node, rect));
        let on = chrome.focused == Some(node);
        veil.rect_filled(
            rect,
            rect.height() * 0.5,
            if response.hovered() {
                ink(tone::body())
            } else {
                text_primary()
            },
        );
        if on {
            veil.rect_stroke(
                rect.expand(3.0),
                rect.height() * 0.5 + 3.0,
                egui::Stroke::new(2.0, text_primary()),
                egui::StrokeKind::Outside,
            );
        }
        let font = crate::fonts::bold(text * 1.05);
        let label_w = crate::flex::text_size(&veil, &row.label, &font).x;
        let icon_side = text * 1.4;
        let span = icon_side + text * 0.7 + label_w;
        let start = rect.center().x - span * 0.5;
        icon(
            row.icon.as_deref().unwrap_or("AudioLines"),
            Pos2::new(start + icon_side * 0.5, rect.center().y),
            icon_side,
            Color32::BLACK,
        );
        veil.text(
            Pos2::new(start + icon_side + text * 0.7, rect.center().y),
            Align2::LEFT_CENTER,
            &row.label,
            font,
            Color32::BLACK,
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    y += auto_h;
    if let Some(hint) = hint {
        let mut job = egui::text::LayoutJob::simple(
            hint.to_owned(),
            FontId::proportional(text * 0.88),
            ink(tone::muted()),
            right - left,
        );
        job.halign = egui::Align::Center;
        job.wrap.max_rows = 2;
        let galley = veil.layout_job(job);
        veil.galley(Pos2::new(left, y + text * 0.55), galley, ink(tone::muted()));
        y += hint_h;
    }
    y += gap * 0.5;

    veil.line_segment(
        [Pos2::new(left, y), Pos2::new(right, y)],
        egui::Stroke::new(stroke::hairline(), tokens.border),
    );
    y += gap * 0.5;

    let mut drawn_choices = 0;
    for (index, row) in panel.rows.iter().enumerate() {
        let Some(role) = row.sync else { continue };
        let (row_h, inset) = match role {
            SyncRole::Method => (method_h, 0.0),
            SyncRole::Choice if drawn_choices < choices => {
                drawn_choices += 1;
                (choice_h, text * 3.0)
            }
            _ => continue,
        };
        let rect = Rect::from_min_size(
            Pos2::new(left - 6.0, y),
            Vec2::new(right - left + 12.0, row_h),
        );
        y += row_h;
        let node = node_of(index);
        let response = ui.interact(rect, Id::new(("fluxa-sync-row", node)), Sense::click());
        layout.focusable.push((node, rect));
        let on = chrome.focused == Some(node);
        let fill = if on {
            text_primary()
        } else if role == SyncRole::Choice && row.selected {
            ink(tone::soft())
        } else if response.hovered() {
            ink(tone::faint())
        } else {
            Color32::TRANSPARENT
        };
        veil.rect_filled(rect.shrink2(Vec2::new(0.0, 2.0)), text * 0.9, fill);
        let strong = if on { Color32::BLACK } else { text_primary() };
        let soft = if on {
            Color32::from_black_alpha(150)
        } else {
            ink(tone::muted())
        };
        let mut edge = rect.right() - text * 0.7;
        if role == SyncRole::Method {
            let name = if row.chevron {
                "ChevronRight"
            } else if row.selected {
                "ChevronUp"
            } else {
                "ChevronDown"
            };
            icon(
                name,
                Pos2::new(edge - text * 0.7, rect.center().y),
                text * 1.4,
                soft,
            );
            if let Some(name) = row.icon.as_deref() {
                icon(
                    name,
                    Pos2::new(rect.left() + text * 1.2, rect.center().y),
                    text * 1.4,
                    if on {
                        Color32::BLACK
                    } else {
                        ink(tone::subtle())
                    },
                );
            }
            let text_left = rect.left() + text * 3.0;
            veil.text(
                Pos2::new(text_left, rect.center().y - text * 0.62),
                Align2::LEFT_CENTER,
                &row.label,
                crate::fonts::bold(text * 1.03),
                strong,
            );
            if let Some(detail) = row.detail.as_deref() {
                let font = FontId::proportional(text * 0.88);
                veil.text(
                    Pos2::new(text_left, rect.center().y + text * 0.72),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &veil,
                        detail,
                        &font,
                        (edge - text * 3.0 - text_left).max(0.0),
                    ),
                    font,
                    soft,
                );
            }
        } else {
            if row.selected {
                icon(
                    "Check",
                    Pos2::new(edge - text * 0.6, rect.center().y),
                    text * 1.25,
                    strong,
                );
                edge -= text * 2.0;
            }
            let font = FontId::proportional(text * 0.98);
            veil.text(
                Pos2::new(rect.left() + inset, rect.center().y),
                Align2::LEFT_CENTER,
                truncate_to_width(
                    &veil,
                    &row.label,
                    &font,
                    (edge - rect.left() - inset).max(0.0),
                ),
                font,
                if row.selected || on { strong } else { ink(215) },
            );
        }
        if response.clicked() {
            layout.activated = Some(node);
        }
    }

    if slide < 1.0 && sheet {
        let offset = Vec2::new(0.0, (1.0 - slide) * card.height());
        chrome.context.graphics_mut(|graphics| {
            let list = graphics.entry(layer);
            let last = list.next_idx();
            list.transform_range(
                first,
                last,
                egui::emath::TSTransform::from_translation(offset),
            );
        });
    }
}

fn find_row(panel: &PlayerPanel, role: SyncRole) -> Option<&PanelRow> {
    panel.rows.iter().find(|row| row.sync == Some(role))
}
