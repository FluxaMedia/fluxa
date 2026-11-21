use crate::contract::{contract, valid_id};
use crate::extension::{Extension, ExtensionKind, parse_extension};
use crate::layout::{FormFactor, Node, validate_node};
use crate::resolve::{ResolveCache, ResolveError, Resolved, resolve};
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
    #[serde(default)]
    pub forms: Vec<FormFactor>,
    #[serde(default)]
    pub canvas: BTreeMap<String, f32>,
    #[serde(default)]
    pub constants: Option<String>,
    #[serde(default)]
    pub languages: BTreeMap<String, String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub pages: BTreeMap<String, String>,
    #[serde(default)]
    pub nav: Vec<NavEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavEntry {
    pub page: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Script {
    pub extension: Extension,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct Skin {
    pub manifest: Manifest,
    pub dir: PathBuf,
    theme_css: Option<String>,
    screens: BTreeMap<String, Node>,
    includes: BTreeMap<String, Node>,
    constants: BTreeMap<String, f32>,
    strings: BTreeMap<String, BTreeMap<String, String>>,
    scripts: Vec<Script>,
    elements: Vec<crate::blocks::Element>,
}

impl Skin {
    pub fn scripts(&self) -> &[Script] {
        &self.scripts
    }

    pub fn script_hosts(&self) -> Vec<String> {
        let mut hosts: Vec<String> = self
            .scripts
            .iter()
            .flat_map(|script| script.extension.network_hosts.iter().cloned())
            .collect();
        hosts.sort();
        hosts.dedup();
        hosts
    }

    pub fn script_digest(&self) -> Option<String> {
        if self.scripts.is_empty() {
            return None;
        }
        let mut hash: u64 = 0xcbf29ce484222325;
        let mut feed = |text: &str| {
            for byte in text.bytes().chain([0]) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        };
        feed(&self.manifest.id);
        feed(&self.manifest.version);
        for script in &self.scripts {
            feed(&script.extension.id);
            script
                .extension
                .network_hosts
                .iter()
                .for_each(|host| feed(host));
            feed(&script.source);
        }
        Some(format!("{hash:016x}"))
    }

    pub fn strings(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.strings
    }

    pub fn constants(&self) -> &BTreeMap<String, f32> {
        &self.constants
    }

    pub fn canvas(&self, form: FormFactor) -> Option<f32> {
        let name = match form {
            FormFactor::Mobile => "mobile",
            FormFactor::Desktop => "desktop",
            FormFactor::Tv => "tv",
        };
        self.manifest
            .canvas
            .get(name)
            .copied()
            .filter(|width| *width > 0.0)
    }

    pub fn supports(&self, form: FormFactor) -> bool {
        self.manifest.forms.is_empty() || self.manifest.forms.contains(&form)
    }

    pub fn theme(&self) -> Option<&str> {
        self.theme_css.as_deref()
    }

    pub fn grafted(&self, base: &Skin) -> Skin {
        let mut skin = base.clone();
        for element in &self.elements {
            for node in skin.screens.values_mut().chain(skin.includes.values_mut()) {
                crate::blocks::graft(node, element);
            }
        }
        skin
    }

    pub fn has_elements(&self) -> bool {
        !self.elements.is_empty()
    }

    pub fn replaces(&self, screen: &str) -> bool {
        self.screens.contains_key(screen)
    }

    pub fn resolve(
        &self,
        screen: &str,
        data: &Value,
        cache: &mut ResolveCache,
    ) -> Option<Result<Vec<Resolved>, ResolveError>> {
        let root = self.screens.get(screen)?;
        Some(resolve(root, &self.includes, data, cache))
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

trait Files {
    fn read(&self, relative: &str) -> Result<String, String>;
}

struct Folder<'a>(&'a Path);

impl Files for Folder<'_> {
    fn read(&self, relative: &str) -> Result<String, String> {
        let path = safe_path(self.0, relative)?;
        let size = std::fs::metadata(&path)
            .map_err(|error| format!("{relative}: {error}"))?
            .len();
        if size > FILE_LIMIT {
            return Err(format!("{relative}: larger than {FILE_LIMIT} bytes"));
        }
        std::fs::read_to_string(&path).map_err(|error| format!("{relative}: {error}"))
    }
}

struct Zipped {
    path: PathBuf,
    prefix: String,
}

impl Zipped {
    fn open(path: &Path) -> Result<Self, String> {
        let file =
            std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let archive =
            zip::ZipArchive::new(file).map_err(|error| format!("{}: {error}", path.display()))?;
        let names: Vec<String> = archive
            .file_names()
            .filter_map(|name| name.ok().map(|name| name.into_owned()))
            .collect();
        let prefix = ["manifest.json", "skin.json"]
            .into_iter()
            .find_map(|manifest| {
                names.iter().find_map(|name| {
                    let head = name.strip_suffix(manifest)?;
                    (head.is_empty() || (head.ends_with('/') && head.matches('/').count() == 1))
                        .then(|| head.to_owned())
                })
            })
            .ok_or_else(|| format!("{}: no manifest.json in archive", path.display()))?;
        Ok(Self {
            path: path.to_owned(),
            prefix,
        })
    }
}

impl Files for Zipped {
    fn read(&self, relative: &str) -> Result<String, String> {
        use std::io::Read;
        safe_path(Path::new(""), relative)?;
        let file =
            std::fs::File::open(&self.path).map_err(|error| format!("{relative}: {error}"))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|error| format!("{relative}: {error}"))?;
        let mut entry = archive
            .by_name(&format!("{}{relative}", self.prefix))
            .map_err(|error| format!("{relative}: {error}"))?;
        if entry.size() > FILE_LIMIT {
            return Err(format!("{relative}: larger than {FILE_LIMIT} bytes"));
        }
        let mut text = String::new();
        entry
            .take(FILE_LIMIT)
            .read_to_string(&mut text)
            .map_err(|error| format!("{relative}: {error}"))?;
        Ok(text)
    }
}

struct Embedded<'a>(&'a [(&'a str, &'a str)]);

impl Files for Embedded<'_> {
    fn read(&self, relative: &str) -> Result<String, String> {
        self.0
            .iter()
            .find(|(name, _)| *name == relative)
            .map(|(_, source)| (*source).to_owned())
            .ok_or_else(|| format!("{relative}: missing"))
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(files: &dyn Files, relative: &str) -> Result<T, String> {
    let source = files.read(relative)?;
    serde_json::from_str(&source).map_err(|error| format!("{relative}: {error}"))
}

pub fn load_skin(dir: &Path) -> Result<Skin, Vec<String>> {
    load(&Folder(dir), dir)
}

pub fn load_zip(path: &Path) -> Result<Skin, Vec<String>> {
    let files = Zipped::open(path).map_err(|error| vec![error])?;
    load(&files, path)
}

pub fn install(source: &Path, skins: &Path) -> Result<String, Vec<String>> {
    let skin = if source.is_file() {
        load_zip(source)?
    } else {
        load_skin(source)?
    };
    let id = skin.manifest.id.clone();
    std::fs::create_dir_all(skins).map_err(|error| vec![error.to_string()])?;
    if source.is_file() {
        std::fs::copy(source, skins.join(format!("{id}.zip")))
            .map_err(|error| vec![error.to_string()])?;
    } else {
        copy_tree(source, &skins.join(&id)).map_err(|error| vec![error.to_string()])?;
    }
    Ok(id)
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if entry.file_type()?.is_file() && entry.metadata()?.len() <= FILE_LIMIT {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

pub fn load_embedded(files: &[(&str, &str)]) -> Result<Skin, Vec<String>> {
    load(&Embedded(files), Path::new(""))
}

fn load(files: &dyn Files, dir: &Path) -> Result<Skin, Vec<String>> {
    let mut manifest: Manifest = read_json(files, "manifest.json")
        .or_else(|_| read_json(files, "skin.json"))
        .map_err(|error| vec![error])?;
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

    for name in manifest.canvas.keys() {
        if !["mobile", "desktop", "tv"].contains(&name.as_str()) {
            problems.push(format!("canvas {name:?} must be mobile, desktop or tv"));
        }
    }
    let mut constants = BTreeMap::new();
    if let Some(file) = &manifest.constants {
        match read_json::<BTreeMap<String, f32>>(files, file) {
            Ok(found) => constants = found,
            Err(error) => problems.push(error),
        }
    }

    let mut strings = BTreeMap::new();
    for (code, file) in &manifest.languages {
        match read_json::<BTreeMap<String, Value>>(files, file) {
            Ok(found) => {
                let table = found
                    .into_iter()
                    .filter_map(|(key, value)| Some((key, value.as_str()?.to_owned())))
                    .collect();
                strings.insert(code.clone(), table);
            }
            Err(error) => problems.push(error),
        }
    }

    let mut scripts = Vec::new();
    for file in &manifest.extensions {
        let loaded = files.read(file).and_then(|source| {
            let extension = parse_extension(&source).map_err(|issues| issues.join("; "))?;
            if extension.kind != ExtensionKind::Script {
                return Err(format!("{file}: only script extensions load from a skin"));
            }
            let base = file.rsplit_once('/').map_or("", |(head, _)| head);
            let entry = if base.is_empty() {
                extension.entry.clone()
            } else {
                format!("{base}/{}", extension.entry)
            };
            let source = files.read(&entry)?;
            Ok(Script { extension, source })
        });
        match loaded {
            Ok(script) => scripts.push(script),
            Err(error) => problems.push(error),
        }
    }

    let mut includes = BTreeMap::new();
    for file in &manifest.includes {
        match read_json::<BTreeMap<String, Node>>(files, file) {
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

    for entry in &manifest.nav {
        if !manifest.pages.contains_key(&entry.page) {
            problems.push(format!("nav entry points at unknown page {:?}", entry.page));
        }
    }
    let mut screens = BTreeMap::new();
    for (name, file) in &manifest.pages {
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
            || contract().screens.contains(name)
        {
            problems.push(format!("invalid page name {name:?}"));
            continue;
        }
        match read_json::<Node>(files, file) {
            Ok(node) => {
                validate_node(&node, &includes, &format!("page {name}"), &mut problems);
                screens.insert(format!("page.{name}"), node);
            }
            Err(error) => problems.push(error),
        }
    }
    for (name, file) in &manifest.screens {
        if !contract().screens.contains(name) {
            problems.push(format!("unknown screen {name:?}"));
            continue;
        }
        match read_json::<Node>(files, file) {
            Ok(node) => {
                validate_node(&node, &includes, &format!("screen {name}"), &mut problems);
                screens.insert(name.clone(), node);
            }
            Err(error) => problems.push(error),
        }
    }
    let mut theme_css = None;
    if let Some(theme) = &manifest.theme {
        match files.read(theme) {
            Ok(css) => theme_css = Some(css),
            Err(_) => problems.push(format!("theme {theme:?} is missing")),
        }
    }

    let mut elements = Vec::new();
    if let Some(css) = theme_css.take() {
        match crate::blocks::extract(&css) {
            Ok(blocks) => {
                theme_css = Some(blocks.css);
                elements = blocks.elements;
                for page in blocks.pages {
                    validate_node(
                        &page.node,
                        &includes,
                        &format!("page {}", page.name),
                        &mut problems,
                    );
                    if let Some(label) = page.label {
                        manifest.nav.push(NavEntry {
                            page: page.name.clone(),
                            label,
                            icon: page.icon,
                        });
                    }
                    screens.insert(format!("page.{}", page.name), page.node);
                }
                for element in &elements {
                    validate_node(&element.node, &includes, "element", &mut problems);
                }
            }
            Err(errors) => problems.extend(errors),
        }
    }

    if problems.is_empty() {
        Ok(Skin {
            elements,
            manifest,
            dir: dir.to_owned(),
            theme_css,
            screens,
            includes,
            constants,
            strings,
            scripts,
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
        .filter(|entry| {
            let path = entry.path();
            path.join("manifest.json").is_file()
                || path.join("skin.json").is_file()
                || (path.is_file() && path.extension().is_some_and(|ext| ext == "zip"))
        })
        .collect();
    folders.sort_by_key(|entry| entry.file_name());
    let mut skins: Vec<Skin> = Vec::new();
    let mut problems = Vec::new();
    for entry in folders.into_iter().take(SKIN_LIMIT) {
        let folder = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let loaded = if path.is_file() {
            load_zip(&path)
        } else {
            load_skin(&path)
        };
        match loaded {
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

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
        "schemaVersion": 1, "id": "agenda", "name": "Agenda", "version": "1.0.0",
        "screens": { "calendar": "screens/calendar.json" },
        "extensions": ["ext/extension.json"]
    }"#;
    const EXTENSION: &str = r#"{
        "schemaVersion": 1, "id": "agenda", "name": "Agenda", "version": "1.0.0",
        "kind": "script", "entry": "script.js",
        "permissions": ["network"], "networkHosts": ["example.com"]
    }"#;
    const SCREEN: &str = r#"{
        "type": "column",
        "children": [{
            "each": "ext.agenda.cells", "as": "cell",
            "template": { "type": "text", "text": "{cell.label}" }
        }]
    }"#;

    #[test]
    fn script_extension_data_reaches_the_screen() {
        let skin = load_embedded(&[
            ("manifest.json", MANIFEST),
            ("ext/extension.json", EXTENSION),
            ("ext/script.js", "class Provider {}"),
            ("screens/calendar.json", SCREEN),
        ])
        .unwrap();
        assert_eq!(skin.scripts()[0].extension.id, "agenda");
        let data = serde_json::json!({"ext": {"agenda": {"cells": [
            {"label": "Mon"}, {"label": "Tue"}
        ]}}});
        let tree = skin
            .resolve("calendar", &data, &mut ResolveCache::new())
            .unwrap()
            .unwrap();
        let labels: Vec<_> = tree[0]
            .children
            .iter()
            .filter_map(|node| node.text.as_deref())
            .collect();
        assert_eq!(labels, ["Mon", "Tue"]);
    }

    #[test]
    fn missing_script_entry_is_reported() {
        let problems = load_embedded(&[
            ("manifest.json", MANIFEST),
            ("ext/extension.json", EXTENSION),
            ("screens/calendar.json", SCREEN),
        ])
        .unwrap_err();
        assert!(problems.iter().any(|problem| problem.contains("script.js")));
    }

    #[test]
    fn example_agenda_skin_loads_with_its_page_and_nav_entry() {
        let skin = load_skin(std::path::Path::new("../../shared/skins/examples/agenda")).unwrap();
        assert!(skin.replaces("page.agenda"));
        assert_eq!(skin.manifest.nav[0].page, "agenda");
        assert_eq!(skin.scripts().len(), 1);
    }

    #[test]
    fn nav_entry_without_a_page_is_reported() {
        let problems = load_embedded(&[(
            "manifest.json",
            r#"{"schemaVersion":1,"id":"x","name":"X","version":"1.0.0","nav":[{"page":"gone","label":"Gone"}]}"#,
        )])
        .unwrap_err();
        assert!(problems.iter().any(|problem| problem.contains("gone")));
    }

    #[test]
    fn css_element_is_grafted_into_the_base_skin_and_page_opens() {
        let base = load_embedded(&[
            (
                "manifest.json",
                r#"{"schemaVersion":1,"id":"base","name":"B","version":"1.0.0","screens":{"home":"screens/home.json"}}"#,
            ),
            (
                "screens/home.json",
                r#"{"type":"column","class":"shelf","children":[{"type":"text","text":"one"}]}"#,
            ),
        ])
        .unwrap();
        let skin = load_embedded(&[
            (
                "manifest.json",
                r#"{"schemaVersion":1,"id":"mine","name":"M","version":"1.0.0","theme":"style.css"}"#,
            ),
            (
                "style.css",
                r#".shelf { gap: 4px; }
                @element extra { parent: .shelf; type: text; text: "two"; }
                @page agenda { nav-label: "Agenda"; }
                @element row { parent: .page-agenda; type: text; text: "hello"; }"#,
            ),
        ])
        .unwrap();
        assert_eq!(skin.manifest.nav[0].page, "agenda");
        assert!(skin.replaces("page.agenda"));
        assert!(!skin.theme().unwrap().contains("@element"));
        let grafted = skin.grafted(&base);
        let tree = grafted
            .resolve("home", &serde_json::json!({}), &mut ResolveCache::new())
            .unwrap()
            .unwrap();
        let texts: Vec<_> = tree[0]
            .children
            .iter()
            .filter_map(|node| node.text.as_deref())
            .collect();
        assert_eq!(texts, ["one", "two"]);
        let own = skin.grafted(&skin);
        let page = own
            .resolve(
                "page.agenda",
                &serde_json::json!({}),
                &mut ResolveCache::new(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(page[0].children[0].text.as_deref(), Some("hello"));
    }
}
