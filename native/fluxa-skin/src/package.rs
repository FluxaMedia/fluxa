use crate::contract::{contract, valid_id};
use crate::layout::{FormFactor, Node, validate_node};
use crate::resolve::{ResolveError, Resolved, resolve};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

const FILE_LIMIT: u64 = 1024 * 1024;
const SKIN_LIMIT: usize = 32;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub screens: BTreeMap<String, String>,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Skin {
    pub manifest: Manifest,
    pub dir: PathBuf,
    screens: BTreeMap<String, Node>,
    includes: BTreeMap<String, Node>,
}

impl Skin {
    pub fn replaces(&self, screen: &str) -> bool {
        self.screens.contains_key(screen)
    }

    pub fn resolve(
        &self,
        screen: &str,
        data: &Value,
        form: FormFactor,
    ) -> Option<Result<Vec<Resolved>, ResolveError>> {
        let root = self.screens.get(screen)?;
        Some(resolve(root, &self.includes, data, form))
    }
}

fn safe_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    let clean = path
        .components()
        .all(|part| matches!(part, Component::Normal(_)));
    if relative.is_empty() || !clean {
        return Err(format!(
            "{relative:?} must be a relative path inside the skin"
        ));
    }
    Ok(root.join(path))
}

fn read_json<T: for<'de> Deserialize<'de>>(root: &Path, relative: &str) -> Result<T, String> {
    let path = safe_path(root, relative)?;
    let size = std::fs::metadata(&path)
        .map_err(|error| format!("{relative}: {error}"))?
        .len();
    if size > FILE_LIMIT {
        return Err(format!("{relative}: larger than {FILE_LIMIT} bytes"));
    }
    let source = std::fs::read_to_string(&path).map_err(|error| format!("{relative}: {error}"))?;
    serde_json::from_str(&source).map_err(|error| format!("{relative}: {error}"))
}

pub fn load_skin(dir: &Path) -> Result<Skin, Vec<String>> {
    let manifest: Manifest = read_json(dir, "skin.json").map_err(|error| vec![error])?;
    let mut problems = Vec::new();
    if manifest.schema_version != 1 {
        problems.push(format!(
            "schemaVersion {} is not supported",
            manifest.schema_version
        ));
    }
    if !valid_id(&manifest.id) {
        problems.push(format!(
            "id {:?} may use letters, digits, - and _",
            manifest.id
        ));
    }

    let mut includes = BTreeMap::new();
    for file in &manifest.includes {
        match read_json::<BTreeMap<String, Node>>(dir, file) {
            Ok(found) => {
                for (name, node) in found {
                    if includes.insert(name.clone(), node).is_some() {
                        problems.push(format!("{file}: include {name:?} is defined twice"));
                    }
                }
            }
            Err(error) => problems.push(error),
        }
    }
    for (name, node) in &includes {
        validate_node(node, &includes, &format!("include {name}"), &mut problems);
    }

    let mut screens = BTreeMap::new();
    for (name, file) in &manifest.screens {
        if !contract().screens.contains(name) {
            problems.push(format!("unknown screen {name:?}"));
            continue;
        }
        match read_json::<Node>(dir, file) {
            Ok(node) => {
                validate_node(&node, &includes, &format!("screen {name}"), &mut problems);
                screens.insert(name.clone(), node);
            }
            Err(error) => problems.push(error),
        }
    }
    if let Some(theme) = &manifest.theme
        && safe_path(dir, theme)
            .map(|path| !path.is_file())
            .unwrap_or(true)
    {
        problems.push(format!("theme {theme:?} is missing"));
    }

    if problems.is_empty() {
        Ok(Skin {
            manifest,
            dir: dir.to_owned(),
            screens,
            includes,
        })
    } else {
        Err(problems)
    }
}

pub fn load_skins(directory: &Path) -> (Vec<Skin>, Vec<String>) {
    let mut folders: Vec<_> = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("skin.json").is_file())
        .collect();
    folders.sort_by_key(|entry| entry.file_name());
    let mut skins: Vec<Skin> = Vec::new();
    let mut problems = Vec::new();
    for entry in folders.into_iter().take(SKIN_LIMIT) {
        let folder = entry.file_name().to_string_lossy().into_owned();
        match load_skin(&entry.path()) {
            Ok(skin)
                if skins
                    .iter()
                    .any(|other| other.manifest.id == skin.manifest.id) =>
            {
                problems.push(format!(
                    "{folder}: id {:?} is already used",
                    skin.manifest.id
                ));
            }
            Ok(skin) => skins.push(skin),
            Err(found) => {
                problems.extend(found.into_iter().map(|error| format!("{folder}: {error}")))
            }
        }
    }
    (skins, problems)
}
