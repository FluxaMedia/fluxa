use ego_tree::NodeId;
use scraper::{ElementRef, Html, Selector};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const MAX_DOCUMENTS: usize = 512;
const MAX_HTML_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_HTML_BYTES: usize = 48 * 1024 * 1024;
const MAX_NESTING: usize = 2048;
const MAX_ELEMENT_HANDLES: usize = 500_000;

#[derive(Default)]
struct DomState {
    documents: HashMap<String, Html>,
    elements: HashMap<String, (String, NodeId)>,
    html_bytes: usize,
    counter: u64,
}

impl DomState {
    fn next_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

pub struct DomBridge {
    state: RefCell<DomState>,
}

impl DomBridge {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: RefCell::new(DomState::default()),
        })
    }

    pub fn load(&self, html: String) -> String {
        let mut state = self.state.borrow_mut();
        if html.len() > MAX_HTML_BYTES
            || nesting_exceeds(&html, MAX_NESTING)
            || state.documents.len() >= MAX_DOCUMENTS
            || state.html_bytes + html.len() > MAX_TOTAL_HTML_BYTES
        {
            return "__NONE__".to_string();
        }
        state.html_bytes += html.len();
        let doc_id = format!("doc_{}", state.next_id());
        state
            .documents
            .insert(doc_id.clone(), Html::parse_document(&html));
        doc_id
    }

    pub fn select(&self, doc_id: String, selector: String) -> String {
        let mut state = self.state.borrow_mut();
        if selector.is_empty() {
            return "[]".into();
        }
        let ids: Vec<NodeId> = match state.documents.get(&doc_id) {
            Some(doc) => run_selector(doc, &selector)
                .iter()
                .map(|el| el.id())
                .collect(),
            None => return "[]".into(),
        };
        cache_and_encode(&mut state, &doc_id, ids)
    }

    pub fn find(&self, doc_id: String, element_id: String, selector: String) -> String {
        let mut state = self.state.borrow_mut();
        let scope = match state.elements.get(&element_id) {
            Some((_, node_id)) => *node_id,
            None => return "[]".into(),
        };
        let ids: Vec<NodeId> = match state.documents.get(&doc_id) {
            Some(doc) => run_selector_within(doc, scope, &selector)
                .iter()
                .map(|el| el.id())
                .collect(),
            None => return "[]".into(),
        };
        cache_and_encode(&mut state, &doc_id, ids)
    }

    pub fn text(&self, element_ids_csv: String) -> String {
        let state = self.state.borrow();
        element_ids_csv
            .split(',')
            .filter(|id| !id.is_empty())
            .filter_map(|id| resolve(&state, id))
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn html(&self, doc_id: String, element_id: String) -> String {
        let state = self.state.borrow();
        if element_id.is_empty() {
            state
                .documents
                .get(&doc_id)
                .map(|d| d.html())
                .unwrap_or_default()
        } else {
            resolve(&state, &element_id)
                .map(|el| el.inner_html())
                .unwrap_or_default()
        }
    }

    pub fn inner_html(&self, element_id: String) -> String {
        let state = self.state.borrow();
        resolve(&state, &element_id)
            .map(|el| el.inner_html())
            .unwrap_or_default()
    }

    pub fn attr(&self, element_id: String, attr_name: String) -> String {
        let state = self.state.borrow();
        match resolve(&state, &element_id).and_then(|el| el.attr(&attr_name)) {
            Some(value) if !value.is_empty() => value.to_string(),
            _ => "__UNDEFINED__".to_string(),
        }
    }

    pub fn next(&self, doc_id: String, element_id: String) -> String {
        self.sibling(&doc_id, &element_id, true)
    }

    pub fn prev(&self, doc_id: String, element_id: String) -> String {
        self.sibling(&doc_id, &element_id, false)
    }

    fn sibling(&self, doc_id: &str, element_id: &str, forward: bool) -> String {
        let mut state = self.state.borrow_mut();
        let start = match state.elements.get(element_id) {
            Some((_, node_id)) => *node_id,
            None => return "__NONE__".into(),
        };

        let found = match state.documents.get(doc_id) {
            Some(doc) => next_element_sibling_id(doc, start, forward),
            None => return "__NONE__".into(),
        };

        match found {
            Some(node_id) => {
                let element_id = format!("{doc_id}:{}", state.next_id());
                state
                    .elements
                    .insert(element_id.clone(), (doc_id.to_string(), node_id));
                element_id
            }
            None => "__NONE__".to_string(),
        }
    }
}

const VOID_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn nesting_exceeds(html: &str, limit: usize) -> bool {
    let bytes = html.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while let Some(offset) = bytes[i..].iter().position(|&b| b == b'<') {
        i += offset + 1;
        let Some(&first) = bytes.get(i) else { break };
        if first == b'/' {
            depth = depth.saturating_sub(1);
        } else if first.is_ascii_alphabetic() {
            let end = bytes[i..]
                .iter()
                .position(|b| b.is_ascii_whitespace() || matches!(b, b'>' | b'/'))
                .map_or(bytes.len(), |n| i + n);
            let name = String::from_utf8_lossy(&bytes[i..end]).to_ascii_lowercase();
            let self_closed = bytes[end..]
                .iter()
                .position(|&b| b == b'>')
                .is_some_and(|n| n > 0 && bytes[end + n - 1] == b'/');
            if !self_closed && !VOID_TAGS.contains(&name.as_str()) {
                depth += 1;
                if depth > limit {
                    return true;
                }
            }
        }
    }
    false
}

fn resolve<'a>(state: &'a DomState, element_id: &str) -> Option<ElementRef<'a>> {
    let (doc_id, node_id) = state.elements.get(element_id)?;
    let doc = state.documents.get(doc_id)?;
    ElementRef::wrap(doc.tree.get(*node_id)?)
}

fn next_element_sibling_id(doc: &Html, from: NodeId, forward: bool) -> Option<NodeId> {
    let mut cursor = doc.tree.get(from);
    loop {
        let node_ref = cursor?;
        let sib = if forward {
            node_ref.next_sibling()
        } else {
            node_ref.prev_sibling()
        };
        let sib = sib?;
        let sib_id = sib.id();
        if ElementRef::wrap(sib).is_some() {
            return Some(sib_id);
        }
        cursor = doc.tree.get(sib_id);
    }
}

fn split_contains(selector: &str) -> (String, Option<String>) {
    let Some(start) = selector.find(":contains(") else {
        return (selector.to_string(), None);
    };
    let after = &selector[start + ":contains(".len()..];
    let Some(end) = after.find(')') else {
        return (selector.to_string(), None);
    };
    let needle = after[..end]
        .trim_matches(|c| c == '\'' || c == '"')
        .to_string();
    let mut base = String::new();
    base.push_str(&selector[..start]);
    base.push_str(&after[end + 1..]);
    let base = base.trim();
    let base = if base.is_empty() {
        "*".to_string()
    } else {
        base.to_string()
    };
    (base, Some(needle))
}

fn run_selector<'a>(doc: &'a Html, selector: &str) -> Vec<ElementRef<'a>> {
    let (base, needle) = split_contains(selector);
    let Ok(sel) = Selector::parse(&base) else {
        return Vec::new();
    };
    let mut results: Vec<ElementRef<'a>> = doc.select(&sel).collect();
    if let Some(needle) = needle {
        results.retain(|el| el.text().collect::<String>().contains(&needle));
    }
    results
}

fn run_selector_within<'a>(doc: &'a Html, scope: NodeId, selector: &str) -> Vec<ElementRef<'a>> {
    let (base, needle) = split_contains(selector);
    let Ok(sel) = Selector::parse(&base) else {
        return Vec::new();
    };
    let Some(scope_ref) = doc.tree.get(scope).and_then(ElementRef::wrap) else {
        return Vec::new();
    };
    let mut results: Vec<ElementRef<'a>> = scope_ref.select(&sel).collect();
    if let Some(needle) = needle {
        results.retain(|el| el.text().collect::<String>().contains(&needle));
    }
    results
}

fn cache_and_encode(state: &mut DomState, doc_id: &str, ids: Vec<NodeId>) -> String {
    let room = MAX_ELEMENT_HANDLES.saturating_sub(state.elements.len());
    let encoded: Vec<String> = ids
        .into_iter()
        .take(room)
        .map(|node_id| {
            let element_id = format!("{doc_id}:{}", state.next_id());
            state
                .elements
                .insert(element_id.clone(), (doc_id.to_string(), node_id));
            element_id
        })
        .collect();
    serde_json::to_string(&encoded).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeply_nested_html_is_refused_before_parsing() {
        let depth = 200_000;
        let html = format!("{}x{}", "<div>".repeat(depth), "</div>".repeat(depth));
        let dom = DomBridge::new();
        assert_eq!(dom.load(html), "__NONE__");
        let depth = 1500;
        let html = format!("{}x{}", "<div>".repeat(depth), "</div>".repeat(depth));
        let doc = dom.load(html);
        let found: Vec<String> =
            serde_json::from_str(&dom.select(doc.clone(), "div".into())).unwrap();
        assert!(!found.is_empty());
        let _ = dom.text(found[0].clone());
        let _ = dom.html(doc.clone(), found[0].clone());
        let _ = dom.inner_html(found[0].clone());
        let _ = dom.find(doc, found[0].clone(), "div div".into());
    }

    #[test]
    fn documents_are_capped() {
        let dom = DomBridge::new();
        let loaded = (0..MAX_DOCUMENTS + 10)
            .filter(|_| dom.load("<p>a</p>".into()) != "__NONE__")
            .count();
        assert_eq!(loaded, MAX_DOCUMENTS);
    }

    #[test]
    fn total_html_is_budgeted() {
        let dom = DomBridge::new();
        let page = "a".repeat(MAX_HTML_BYTES);
        let loaded = (0..6)
            .filter(|_| dom.load(page.clone()) != "__NONE__")
            .count();
        assert_eq!(loaded, MAX_TOTAL_HTML_BYTES / MAX_HTML_BYTES);
    }

    #[test]
    fn oversized_html_is_refused() {
        let dom = DomBridge::new();
        assert_eq!(dom.load("a".repeat(MAX_HTML_BYTES + 1)), "__NONE__");
    }

    #[test]
    fn element_handles_are_capped() {
        let dom = DomBridge::new();
        let doc = dom.load("<i></i>".repeat(1000));
        for _ in 0..MAX_ELEMENT_HANDLES / 1000 + 5 {
            dom.select(doc.clone(), "i".into());
        }
        assert!(dom.state.borrow().elements.len() <= MAX_ELEMENT_HANDLES);
    }

    #[test]
    fn malformed_selectors_and_ids_return_empty() {
        let dom = DomBridge::new();
        let doc = dom.load("<p>a</p>".into());
        for selector in [
            "",
            "((",
            "p:contains(",
            ":contains()",
            "p >>> q",
            "\\",
            "[",
            "p:nth-child(99999999999999999999)",
        ] {
            assert!(
                dom.select(doc.clone(), selector.into()).starts_with('['),
                "{selector}"
            );
        }
        assert_eq!(dom.select("nope".into(), "p".into()), "[]");
        assert_eq!(dom.text("a,,b,doc_0:99".into()), "");
    }
}
