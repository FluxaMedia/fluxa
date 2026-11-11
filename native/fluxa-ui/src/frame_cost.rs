use super::*;
use std::time::{Duration, Instant};

struct NoAssets;

impl HomeAssets for NoAssets {
    fn background(&self) -> TextureId {
        TextureId::Managed(0)
    }

    fn texture(&mut self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
}

struct Cost {
    first: f32,
    mean: f32,
    p95: f32,
    max: f32,
    repainting: usize,
    last_repaints: bool,
    frames: usize,
}

fn measure(
    viewport: Viewport,
    frames: usize,
    mut draw: impl FnMut(&crate::gfx::Context, &mut NoAssets) -> HomeLayout,
) -> Cost {
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = NoAssets;
    let mut times = Vec::with_capacity(frames);
    let mut first = 0.0;
    let mut repainting = 0;
    let mut last_repaints = false;
    for frame in 0..frames {
        let input = crate::gfx::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(viewport.width, viewport.height),
            )),
            time: Some(frame as f64 / 60.0),
            ..Default::default()
        };
        let started = Instant::now();
        let output = context.run_ui(input, |ui| {
            draw(ui.ctx(), &mut assets);
        });
        let elapsed = started.elapsed().as_secs_f32() * 1000.0;
        if frame == 0 {
            first = elapsed;
        } else {
            times.push(elapsed);
            last_repaints = output.repaint_delay() < Duration::from_millis(500);
            if last_repaints {
                repainting += 1;
            }
        }
        output.drop_without_applying_deltas();
    }
    times.sort_by(f32::total_cmp);
    let steady = times.len();
    Cost {
        first,
        mean: times.iter().sum::<f32>() / steady as f32,
        p95: times[(steady as f32 * 0.95) as usize],
        max: times[steady - 1],
        repainting,
        last_repaints,
        frames: steady,
    }
}

fn report(name: &str, cost: &Cost) {
    println!(
        "{name:<22} first={:6.1}ms mean={:6.2}ms p95={:6.2}ms max={:6.2}ms repaint_requested={}/{} idle_repaints={}",
        cost.first, cost.mean, cost.p95, cost.max, cost.repainting, cost.frames, cost.last_repaints
    );
}

fn busy_home() -> HomeModel {
    home_with_cards(18)
}

fn home_with_cards(per_row: usize) -> HomeModel {
    let mut home = HomeModel::default();
    home.profile_name = Some("Khooly".into());
    home.show_hero_section = true;
    let cards = |row: usize, kind: HomeRowKind| -> Vec<HomeCard> {
        (0..per_row)
            .map(|index| HomeCard {
                id: Some(format!("tt{row}{index}")),
                item_type: Some("movie".into()),
                title: format!("Sample title number {row}-{index}"),
                subtitle: "2024".into(),
                progress: if kind == HomeRowKind::Continue {
                    0.4
                } else {
                    0.0
                },
                row_kind: kind,
                ..Default::default()
            })
            .collect()
    };
    home.rows = (0..8)
        .map(|row| {
            let kind = if row == 0 {
                HomeRowKind::Continue
            } else {
                HomeRowKind::Poster
            };
            HomeRow {
                id: Some(format!("row{row}")),
                title: format!("Catalog row {row}"),
                cards: cards(row, kind),
                kind,
                ..Default::default()
            }
        })
        .collect();
    home
}

#[test]
#[ignore]
fn frame_cost() {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let home = busy_home();
    report(
        "home",
        &measure(viewport, 120, |context, assets| {
            draw_home(context, viewport, &home, assets, None)
        }),
    );
    for section in [0, 1, 4, 8, 13] {
        let mut settings = SettingsModel::default();
        settings.active_section = section;
        settings.section_open = true;
        report(
            &format!("settings/{section}"),
            &measure(viewport, 120, |context, assets| {
                draw_settings(context, viewport, &settings, assets)
            }),
        );
    }
}

#[test]
#[ignore]
fn long_rows_cost() {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    for per_row in [18, 200, 1000] {
        let home = home_with_cards(per_row);
        report(
            &format!("home x{per_row}"),
            &measure(viewport, 120, |context, assets| {
                draw_home(context, viewport, &home, assets, None)
            }),
        );
    }
}
