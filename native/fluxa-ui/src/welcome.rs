use super::*;
use crate::components::InputKind;
use crate::kit::{self, Role, Type, UiKit};

#[derive(Clone, Debug, Default)]
pub struct WelcomeQr {
    pub modules: Vec<bool>,
    pub width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WelcomeModel {
    pub language: String,
    pub email: String,
    pub password: String,
    pub show_password: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub qr: Option<WelcomeQr>,
    pub pair_code: Option<(String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeRequest {
    SignIn,
    Offline,
}

pub fn draw_welcome(
    context: &crate::gfx::Context,
    viewport: Viewport,
    model: &mut WelcomeModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    painter.rect_filled(screen, 0.0, background());
    let metrics = metrics_for_assets(viewport, assets);
    kit::provide(context, metrics);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let wordmark = assets.logo("wordmark");
    let eye = assets.icon(if model.show_password { "EyeOff" } else { "Eye" });

    let area_id = Id::new("fluxa-welcome");
    if viewport.is_tv() {
        crate::components::fit_to_safe_area(context, viewport, area_id);
    }
    crate::gfx::Area::new(area_id)
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(screen.size());
            ui.set_max_size(screen.size());
            if viewport.is_tv() {
                crate::components::unclip_for_fit(ui, screen.size());
                let content = draw_pairing(ui, viewport, model, wordmark, &t, &mut request);
                crate::components::record_fit_content(context, area_id, content);
                return;
            }
            crate::components::pan_area(
                ui,
                "fluxa-welcome-scroll",
                |area| area.auto_shrink([false, false]),
                |ui| draw_login(ui, viewport, model, wordmark, eye, &t, &mut request),
            );
        });

    HomeLayout {
        welcome: request,
        ..HomeLayout::default()
    }
}

fn draw_login(
    ui: &mut crate::gfx::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(crate::gfx::TextureId, Vec2)>,
    eye: Option<crate::gfx::TextureId>,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let theme = ui.tokens();
    let top = ((viewport.height - 620.0) * 0.5).max(32.0);
    let busy = model.busy;
    let show_password = model.show_password;
    let error = model.error.as_deref().map(|key| t(key));
    let title = t("auth.welcome_back");
    let subtitle = t("auth.nuvio.subtitle");
    let email_label = t("auth.field.email");
    let email_hint = t("auth.placeholder.email");
    let password_label = t("auth.field.password");
    let password_hint = t("auth.placeholder.password_login");
    let sign_in_label = if busy {
        t("auth.nuvio.signing_in")
    } else {
        t("auth.log_in")
    };
    let divider = t("auth.or");
    let offline_label = t("auth.continue_without_account");
    let email_text = &mut model.email;
    let password_text = &mut model.password;
    let email = std::cell::RefCell::new(None);
    let password = std::cell::RefCell::new(None);
    let toggled = std::cell::Cell::new(false);
    let sign_in = std::cell::Cell::new(false);
    let offline = std::cell::Cell::new(false);
    kit::view()
        .column()
        .max_width(theme.welcome_max_width)
        .padding(Space::S16)
        .center_x()
        .show(ui, |ui| {
            let items = vec![
                kit::layout_items::gap(top),
                kit::widget(ui, |ui| kit::wordmark(ui, wordmark, 52.0)),
                kit::layout_items::space(ui, Space::S20),
                kit::widget(ui, |ui| {
                    kit::text(title.clone(), Type::Title).centered().show(ui);
                }),
                kit::layout_items::space(ui, Space::S8),
                kit::widget(ui, |ui| {
                    kit::text(subtitle.clone(), Type::Subtitle)
                        .role(Role::Secondary)
                        .centered()
                        .show(ui);
                }),
                kit::layout_items::space(ui, Space::S24),
                field_label(ui, &email_label),
                kit::widget(ui, |ui| {
                    *email.borrow_mut() = Some(kit::input(&mut *email_text, &email_hint).show(ui));
                }),
                kit::layout_items::space(ui, Space::S24),
                field_label(ui, &password_label),
                kit::widget(ui, |ui| {
                    let field = ui.max_rect();
                    let hit = crate::flex::anchored(
                        field,
                        crate::gfx::Align2::RIGHT_CENTER,
                        Vec2::splat(40.0),
                        8.0,
                    );
                    let kind = if show_password {
                        InputKind::Text
                    } else {
                        InputKind::Password
                    };
                    let eye_clicked = ui.input(|input| {
                        input.pointer().any_click()
                            && input
                                .pointer()
                                .interact_pos()
                                .is_some_and(|pos| hit.contains(pos))
                    });
                    if eye_clicked {
                        ui.ctx().keep_focus_on_click(true);
                    }
                    let response = kit::input(&mut *password_text, &password_hint)
                        .kind(kind)
                        .trailing(56.0)
                        .show(ui);
                    if eye_clicked {
                        ui.ctx().keep_focus_on_click(false);
                    }
                    *password.borrow_mut() = Some(response);
                    let button = ui.interact(
                        hit,
                        Id::new("welcome-show-password"),
                        crate::gfx::Sense::CLICK,
                    );
                    if let Some(texture) = eye {
                        let tint = if button.hovered() {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        };
                        ui.painter().image(
                            texture,
                            crate::flex::anchored(
                                hit,
                                crate::gfx::Align2::CENTER_CENTER,
                                Vec2::splat(20.0),
                                0.0,
                            ),
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            tint,
                        );
                    }
                    if button
                        .on_hover_cursor(crate::gfx::CursorIcon::PointingHand)
                        .clicked()
                    {
                        toggled.set(true);
                    }
                }),
                kit::layout_items::space(ui, Space::S4),
                match error.as_deref() {
                    Some(message) => kit::widget(ui, |ui| components::note(ui, message, *theme)),
                    None => kit::layout_items::gap(0.0),
                },
                kit::layout_items::space(ui, Space::S48),
                kit::widget(ui, |ui| {
                    let response = kit::button(&sign_in_label)
                        .primary()
                        .enabled(!busy)
                        .show(ui);
                    if response.clicked() {
                        sign_in.set(true);
                    }
                }),
                kit::layout_items::space(ui, Space::S20),
                kit::widget(ui, |ui| kit::labeled_divider(ui, &divider)),
                kit::layout_items::space(ui, Space::S20),
                kit::widget(ui, |ui| {
                    if kit::button(&offline_label).show(ui).clicked() {
                        offline.set(true);
                    }
                }),
                kit::layout_items::space(ui, Space::S48),
            ];
            kit::lay(ui, items);
        });
    if let Some(response) = email.borrow().as_ref() {
        keep_in_view(ui, response, viewport.height);
    }
    let password = password.into_inner();
    if let Some(response) = password.as_ref() {
        keep_in_view(ui, response, viewport.height);
    }
    if toggled.get() {
        model.show_password = !model.show_password;
    }
    let submitted = password.is_some_and(|response| response.lost_focus())
        && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
    if sign_in.get() || (submitted && !busy) {
        *request = Some(WelcomeRequest::SignIn);
    }
    if offline.get() {
        *request = Some(WelcomeRequest::Offline);
    }
}

fn keep_in_view(ui: &crate::gfx::Ui, field: &crate::gfx::Response, height: f32) {
    let key = Id::new("welcome-visible-height");
    let changed = ui.data_mut(|data| {
        let before = data.get_temp::<f32>(key);
        data.insert_temp(key, height);
        before != Some(height)
    });
    if field.has_focus() && (changed || field.gained_focus()) {
        let reach = Vec2::new(0.0, 110.0);
        ui.scroll_to_rect(
            Rect::from_min_max(field.rect.min, field.rect.max + reach),
            None,
        );
    }
}

fn field_label<'a>(ui: &mut crate::gfx::Ui, text: &'a str) -> kit::Item<'a> {
    kit::layout_items::column(vec![
        kit::widget(ui, move |ui| {
            kit::text(text, Type::Label).role(Role::Secondary).show(ui);
        }),
        kit::layout_items::space(ui, Space::S8),
    ])
}

fn draw_pairing(
    ui: &mut crate::gfx::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(crate::gfx::TextureId, Vec2)>,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) -> Rect {
    let theme = ui.tokens();
    let qr_side = theme.welcome_qr.of(viewport.height);
    let card_side = qr_side + 64.0;
    let text_width = theme.welcome_text.of(viewport.width);
    let gap = Space::S48.px(&theme);
    let total = card_side + gap + text_width;
    let enter = ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
    let qr = model.qr.as_ref();
    let modules = qr.map_or(&[][..], |qr| qr.modules.as_slice());
    let qr_width = qr.map_or(0, |qr| qr.width);
    let offline_label = t("auth.continue_without_account");
    let title = t("auth.welcome_back");
    let scan = t("auth.scan_to_connect");
    let steps: Vec<String> = [
        "auth.tv.step_scan",
        "auth.tv.step_login",
        "auth.tv.step_return",
    ]
    .iter()
    .map(|key| t(key))
    .collect();
    let pair = model.pair_code.as_ref().map(|(url, code)| {
        (
            t("auth.tv.or_enter_code").replacen("%s", url, 1),
            code.clone(),
        )
    });
    let status = match model.error.as_deref() {
        Some(key) => t(key),
        None => t("auth.login_pending"),
    };
    let offline = std::cell::Cell::new(false);
    let shown = kit::view()
        .column()
        .max_width(total)
        .padding(Space::S24)
        .center_x()
        .show(ui, |ui| {
            let items = vec![
                kit::widget(ui, |ui| {
                    kit::wordmark(ui, wordmark, theme.welcome_wordmark.of(viewport.height))
                }),
                kit::layout_items::space(ui, Space::S20),
                kit::widget(ui, |ui| {
                    kit::view().row().gap(Space::S48).show(ui, |ui| {
                        let columns = vec![
                            kit::widget(ui, |ui| {
                                kit::view().column().max_width(card_side).show(ui, |ui| {
                                    let items = vec![
                                        kit::widget(ui, |ui| {
                                            kit::qr_card(ui, qr_side, modules, qr_width)
                                        }),
                                        kit::layout_items::space(ui, Space::S24),
                                        kit::widget(ui, |ui| {
                                            let response = kit::button(&offline_label).show(ui);
                                            if !kit::probing() {
                                                response
                                                    .clone()
                                                    .focus_node(ui, NODE_WELCOME_OFFLINE);
                                            }
                                            if response.clicked() || enter {
                                                offline.set(true);
                                            }
                                        }),
                                    ];
                                    kit::lay(ui, items);
                                });
                            }),
                            kit::widget(ui, |ui| {
                                kit::view().column().max_width(text_width).show(ui, |ui| {
                                    let mut items = vec![
                                        kit::widget(ui, |ui| {
                                            kit::text(title.clone(), Type::Headline).show(ui);
                                        }),
                                        kit::layout_items::space(ui, Space::S12),
                                        kit::widget(ui, |ui| {
                                            kit::text(scan.clone(), Type::Subtitle)
                                                .role(Role::Secondary)
                                                .show(ui);
                                        }),
                                        kit::layout_items::space(ui, Space::S16),
                                    ];
                                    for (index, step) in steps.iter().enumerate() {
                                        items.push(kit::widget(ui, move |ui| {
                                            kit::step(ui, index + 1, step)
                                        }));
                                        items.push(kit::layout_items::space(ui, Space::S8));
                                    }
                                    if let Some((prompt, code)) = pair.as_ref() {
                                        items.push(kit::widget(ui, |ui| {
                                            kit::text(prompt.clone(), Type::Body)
                                                .role(Role::Secondary)
                                                .show(ui);
                                        }));
                                        items.push(kit::layout_items::space(ui, Space::S8));
                                        items.push(kit::widget(ui, |ui| {
                                            kit::text(code.as_str(), Type::Headline).show(ui);
                                        }));
                                        items.push(kit::layout_items::space(ui, Space::S8));
                                    }
                                    items.push(kit::widget(ui, |ui| {
                                        kit::text(status.clone(), Type::Body)
                                            .role(Role::Muted)
                                            .show(ui);
                                    }));
                                    kit::lay(ui, items);
                                });
                            }),
                        ];
                        kit::lay(ui, columns);
                    });
                }),
            ];
            kit::lay(ui, items);
        });
    if offline.get() {
        *request = Some(WelcomeRequest::Offline);
    }
    shown.response.rect
}
