use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsSection {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

fn pack(
    sections: &[StatsSection],
    columns: usize,
    line: f32,
    gap: f32,
    limit: f32,
) -> Vec<Vec<usize>> {
    let mut packed: Vec<Vec<usize>> = vec![Vec::new()];
    let mut used = line;
    for (index, section) in sections.iter().enumerate() {
        let height = line * (section.rows.len() + 1) as f32;
        let empty = packed.last().is_some_and(Vec::is_empty);
        let spacing = if empty { 0.0 } else { gap };
        if !empty && used + spacing + height > limit && packed.len() < columns {
            packed.push(vec![index]);
            used = height;
        } else {
            packed.last_mut().unwrap().push(index);
            used += spacing + height;
        }
    }
    packed
}

fn column_height(
    sections: &[StatsSection],
    column: &[usize],
    first: bool,
    line: f32,
    gap: f32,
) -> f32 {
    let header = if first { line } else { 0.0 };
    column
        .iter()
        .enumerate()
        .fold(header, |height, (position, index)| {
            let spacing = if position > 0 { gap } else { 0.0 };
            height + spacing + line * (sections[*index].rows.len() + 1) as f32
        })
}

pub(super) fn stats_data(player: &PlayerModel, mobile: bool) -> serde_json::Value {
    if player.stats.is_empty() {
        return serde_json::Value::Null;
    }
    let t = |key: &str| localized(key, &player.language);
    let tokens = super::tokens::PlayerTokens::for_form_factor(if mobile {
        UiFormFactor::Mobile
    } else {
        UiFormFactor::Desktop
    });
    let (budget, max_columns) = (tokens.stats_rows, tokens.stats_columns.max(1));
    let mut columns = pack(&player.stats, 1, 1.0, 0.4, budget);
    for count in 2..=max_columns {
        let tallest = columns
            .iter()
            .enumerate()
            .map(|(at, column)| column_height(&player.stats, column, at == 0, 1.0, 0.4))
            .fold(0.0, f32::max);
        if tallest <= budget {
            break;
        }
        columns = pack(&player.stats, count, 1.0, 0.4, budget);
    }
    let columns: Vec<_> = columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            let sections: Vec<_> = column
                .iter()
                .map(|index| {
                    let section = &player.stats[*index];
                    let rows: Vec<_> = section
                        .rows
                        .iter()
                        .map(|(label, value)| {
                            let (from, to) = match value.split_once(" → ") {
                                Some((from, to)) => (from.to_owned(), to.to_owned()),
                                None => (String::new(), value.clone()),
                            };
                            serde_json::json!({
                                "label": t(label),
                                "from": from,
                                "to": to,
                                "arrow": !from.is_empty(),
                            })
                        })
                        .collect();
                    serde_json::json!({"title": t(&section.title), "rows": rows})
                })
                .collect();
            serde_json::json!({"first": at == 0, "sections": sections})
        })
        .collect();
    serde_json::json!({"summary": t("player.playback_summary"), "columns": columns})
}
