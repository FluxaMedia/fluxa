use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

pub(crate) fn content_warning_url(imdb_id: &str) -> String {
    format!("https://api.tiffara.com/titles/{imdb_id}/parentsGuide")
}

#[derive(Deserialize)]
struct SeverityBreakdown {
    #[serde(rename = "severityLevel", default)]
    severity_level: String,
    #[serde(rename = "voteCount", default)]
    vote_count: i64,
}

#[derive(Deserialize)]
struct ParentsGuideCategory {
    #[serde(default)]
    category: String,
    #[serde(rename = "severityBreakdowns", default)]
    severity_breakdowns: Vec<SeverityBreakdown>,
}

#[derive(Deserialize)]
struct ParentsGuideResponse {
    #[serde(rename = "parentsGuide", default)]
    parents_guide: Vec<ParentsGuideCategory>,
}

#[derive(Deserialize)]
struct ContentWarningLabels {
    nudity: String,
    violence: String,
    profanity: String,
    alcohol: String,
    frightening: String,
    severe: String,
    moderate: String,
    mild: String,
}

#[derive(Deserialize)]
struct ContentWarningsRequest {
    #[serde(rename = "responseJson")]
    response_json: String,
    labels: ContentWarningLabels,
}

fn resolve_category_severity(category: &ParentsGuideCategory) -> Option<String> {
    let breakdowns = &category.severity_breakdowns;
    let dominant = breakdowns
        .iter()
        .filter(|breakdown| breakdown.severity_level.to_lowercase() != "none")
        .max_by_key(|breakdown| breakdown.vote_count)?;
    let none_votes = breakdowns
        .iter()
        .find(|breakdown| breakdown.severity_level.to_lowercase() == "none")
        .map(|breakdown| breakdown.vote_count)
        .unwrap_or(0);
    if dominant.vote_count <= none_votes {
        return None;
    }
    Some(dominant.severity_level.to_lowercase())
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "severe" => 0,
        "moderate" => 1,
        "mild" => 2,
        _ => 3,
    }
}

pub(crate) fn build_content_warnings_json(request_json: &str) -> Option<String> {
    let request: ContentWarningsRequest = serde_json::from_str(request_json).ok()?;
    let response: ParentsGuideResponse = serde_json::from_str(&request.response_json).ok()?;
    let category_map: HashMap<String, &ParentsGuideCategory> = response
        .parents_guide
        .iter()
        .map(|category| (category.category.to_uppercase(), category))
        .collect();

    let categories = [
        ("SEXUAL_CONTENT", &request.labels.nudity),
        ("VIOLENCE", &request.labels.violence),
        ("PROFANITY", &request.labels.profanity),
        ("ALCOHOL_DRUGS", &request.labels.alcohol),
        ("FRIGHTENING_INTENSE_SCENES", &request.labels.frightening),
    ];
    let mut warnings: Vec<(String, String)> = categories
        .iter()
        .filter_map(|(key, label)| {
            let category = category_map.get(*key)?;
            let severity = resolve_category_severity(category)?;
            Some(((*label).clone(), severity))
        })
        .collect();
    warnings.sort_by_key(|(_, severity)| severity_rank(severity));
    warnings.truncate(5);

    let severity_label = |severity: &str| match severity {
        "severe" => request.labels.severe.clone(),
        "moderate" => request.labels.moderate.clone(),
        "mild" => request.labels.mild.clone(),
        other => other.to_string(),
    };
    let result = warnings
        .into_iter()
        .map(|(label, severity)| json!({ "label": label, "severity": severity_label(&severity) }))
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({ "warnings": result })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_warning_url_targets_the_imdb_id() {
        assert_eq!(
            content_warning_url("tt1234567"),
            "https://api.tiffara.com/titles/tt1234567/parentsGuide"
        );
    }

    fn labels() -> serde_json::Value {
        json!({
            "nudity": "Sex & Nudity",
            "violence": "Violence & Gore",
            "profanity": "Profanity",
            "alcohol": "Alcohol, Drugs & Smoking",
            "frightening": "Frightening & Intense Scenes",
            "severe": "Severe",
            "moderate": "Moderate",
            "mild": "Mild",
        })
    }

    #[test]
    fn build_content_warnings_orders_severe_first_and_caps_at_five() {
        let response = json!({
            "parentsGuide": [
                {
                    "category": "VIOLENCE",
                    "severityBreakdowns": [
                        { "severityLevel": "Severe", "voteCount": 10 },
                        { "severityLevel": "None", "voteCount": 1 },
                    ],
                },
                {
                    "category": "PROFANITY",
                    "severityBreakdowns": [
                        { "severityLevel": "Mild", "voteCount": 8 },
                        { "severityLevel": "None", "voteCount": 1 },
                    ],
                },
            ],
        });
        let request = json!({ "responseJson": response.to_string(), "labels": labels() });
        let result: serde_json::Value =
            serde_json::from_str(&build_content_warnings_json(&request.to_string()).unwrap())
                .unwrap();
        let warnings = result["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0]["label"], "Violence & Gore");
        assert_eq!(warnings[0]["severity"], "Severe");
        assert_eq!(warnings[1]["label"], "Profanity");
        assert_eq!(warnings[1]["severity"], "Mild");
    }

    #[test]
    fn build_content_warnings_drops_categories_where_none_dominates() {
        let response = json!({
            "parentsGuide": [
                {
                    "category": "VIOLENCE",
                    "severityBreakdowns": [
                        { "severityLevel": "Mild", "voteCount": 2 },
                        { "severityLevel": "None", "voteCount": 20 },
                    ],
                },
            ],
        });
        let request = json!({ "responseJson": response.to_string(), "labels": labels() });
        let result: serde_json::Value =
            serde_json::from_str(&build_content_warnings_json(&request.to_string()).unwrap())
                .unwrap();
        assert!(result["warnings"].as_array().unwrap().is_empty());
    }
}
