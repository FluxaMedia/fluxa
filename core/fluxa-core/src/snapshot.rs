use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::ops::Index;
use std::sync::{Arc, OnceLock};

static NULL: Value = Value::Null;

#[derive(Debug, Default)]
pub struct HomeSlice {
    rest: Value,
    rows: Vec<Arc<Value>>,
    whole: OnceLock<Value>,
}

impl HomeSlice {
    pub fn new(rest: Value, rows: Vec<Arc<Value>>) -> Self {
        Self {
            rest,
            rows,
            whole: OnceLock::new(),
        }
    }

    pub fn rest(&self) -> &Value {
        &self.rest
    }

    pub fn rows(&self) -> &[Arc<Value>] {
        &self.rows
    }

    pub fn value(&self) -> &Value {
        self.whole.get_or_init(|| {
            let mut whole = self.rest.clone();
            if let Some(object) = whole.as_object_mut() {
                object.insert(
                    "categories".to_owned(),
                    Value::Array(self.rows.iter().map(|row| (**row).clone()).collect()),
                );
            }
            whole
        })
    }

    fn pointer(&self, path: &str) -> Option<&Value> {
        if path.is_empty() {
            return Some(self.value());
        }
        if path == "/categories" || path.starts_with("/categories/") {
            self.value().pointer(path)
        } else {
            self.rest.pointer(path)
        }
    }
}

#[derive(Clone, Debug)]
enum Slice {
    Plain(Arc<Value>),
    Home(Arc<HomeSlice>),
}

impl Slice {
    fn value(&self) -> &Value {
        match self {
            Slice::Plain(value) => value,
            Slice::Home(home) => home.value(),
        }
    }

    fn same(&self, other: &Slice) -> bool {
        match (self, other) {
            (Slice::Plain(a), Slice::Plain(b)) => Arc::ptr_eq(a, b),
            (Slice::Home(a), Slice::Home(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    fn equal(&self, other: &Slice) -> bool {
        self.same(other)
            || match (self, other) {
                (Slice::Plain(a), Slice::Plain(b)) => a == b,
                (Slice::Home(a), Slice::Home(b)) => {
                    a.rest == b.rest
                        && a.rows.len() == b.rows.len()
                        && a.rows
                            .iter()
                            .zip(&b.rows)
                            .all(|(a, b)| Arc::ptr_eq(a, b) || a == b)
                }
                _ => false,
            }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    slices: Arc<BTreeMap<String, Slice>>,
}

impl Snapshot {
    pub fn from_value(root: Value) -> Self {
        let mut slices = BTreeMap::new();
        if let Value::Object(object) = root {
            for (name, value) in object {
                let slice = match (name.as_str(), value) {
                    ("home", Value::Object(mut home)) => {
                        let rows = match home.remove("categories") {
                            Some(Value::Array(rows)) => rows.into_iter().map(Arc::new).collect(),
                            _ => Vec::new(),
                        };
                        Slice::Home(Arc::new(HomeSlice::new(Value::Object(home), rows)))
                    }
                    (_, value) => Slice::Plain(Arc::new(value)),
                };
                slices.insert(name, slice);
            }
        }
        Self {
            slices: Arc::new(slices),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.slices.get(name).map(Slice::value)
    }

    pub fn pointer(&self, path: &str) -> Option<&Value> {
        let rest = path.strip_prefix('/')?;
        let (name, tail) = match rest.find('/') {
            Some(at) => (&rest[..at], &rest[at..]),
            None => (rest, ""),
        };
        match self.slices.get(name)? {
            Slice::Plain(value) => value.pointer(tail),
            Slice::Home(home) => home.pointer(tail),
        }
    }

    pub fn home(&self) -> Option<&HomeSlice> {
        match self.slices.get("home") {
            Some(Slice::Home(home)) => Some(home),
            _ => None,
        }
    }

    pub fn home_rows(&self) -> &[Arc<Value>] {
        self.home().map_or(&[], HomeSlice::rows)
    }

    pub fn slice_shared(&self, other: &Snapshot, name: &str) -> bool {
        match (self.slices.get(name), other.slices.get(name)) {
            (Some(a), Some(b)) => a.same(b),
            (None, None) => true,
            _ => false,
        }
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.slices.keys().map(String::as_str)
    }

    pub fn slice_equal(&self, other: &Snapshot, name: &str) -> bool {
        match (self.slices.get(name), other.slices.get(name)) {
            (Some(a), Some(b)) => a.equal(b),
            (None, None) => true,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, value: Value) {
        Arc::make_mut(&mut self.slices).insert(name.to_owned(), Slice::Plain(Arc::new(value)));
    }

    pub fn set_home(&mut self, home: HomeSlice) {
        Arc::make_mut(&mut self.slices).insert("home".to_owned(), Slice::Home(Arc::new(home)));
    }

    pub fn edit(&mut self, name: &str, edit: impl FnOnce(&mut Value)) -> bool {
        let Some(Slice::Plain(value)) = Arc::make_mut(&mut self.slices).get_mut(name) else {
            return false;
        };
        edit(Arc::make_mut(value));
        true
    }

    pub fn to_value(&self) -> Value {
        Value::Object(
            self.slices
                .iter()
                .map(|(name, slice)| (name.clone(), slice.value().clone()))
                .collect::<Map<_, _>>(),
        )
    }

    pub fn view_except(&self, excluded: &[&str]) -> Value {
        Value::Object(
            self.slices
                .iter()
                .filter(|(name, _)| !excluded.contains(&name.as_str()))
                .map(|(name, slice)| (name.clone(), slice.value().clone()))
                .collect(),
        )
    }

    pub fn view(&self, names: &[&str]) -> Value {
        Value::Object(
            names
                .iter()
                .filter_map(|name| {
                    self.get(name)
                        .map(|value| ((*name).to_owned(), value.clone()))
                })
                .collect(),
        )
    }
}

impl Index<&str> for Snapshot {
    type Output = Value;

    fn index(&self, name: &str) -> &Value {
        self.get(name).unwrap_or(&NULL)
    }
}

impl From<Value> for Snapshot {
    fn from(root: Value) -> Self {
        Snapshot::from_value(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Snapshot {
        Snapshot::from_value(json!({
            "navigation": {"route": "home"},
            "library": {"items": [1, 2]},
            "home": {"isLoading": false, "categories": [{"id": "a"}, {"id": "b"}]},
        }))
    }

    #[test]
    fn home_is_split_into_shared_rows_and_a_remainder() {
        let snapshot = sample();
        assert_eq!(snapshot.home_rows().len(), 2);
        assert_eq!(
            snapshot.home().unwrap().rest(),
            &json!({"isLoading": false})
        );
        assert_eq!(snapshot.pointer("/home/categories/1/id"), Some(&json!("b")));
        assert_eq!(snapshot.pointer("/home/isLoading"), Some(&json!(false)));
        assert_eq!(snapshot["home"]["categories"][0]["id"], "a");
    }

    #[test]
    fn editing_one_slice_leaves_the_others_shared() {
        let before = sample();
        let mut after = before.clone();
        assert!(after.edit("library", |library| library["items"] = json!([1, 2, 3])));
        assert!(!before.slice_shared(&after, "library"));
        assert!(before.slice_shared(&after, "navigation"));
        assert!(before.slice_shared(&after, "home"));
        assert_eq!(before["library"]["items"], json!([1, 2]));
    }

    #[test]
    fn replacing_home_keeps_unchanged_row_arcs() {
        let before = sample();
        let mut rows = before.home_rows().to_vec();
        rows[1] = Arc::new(json!({"id": "b", "items": [1]}));
        let mut after = before.clone();
        after.set_home(HomeSlice::new(json!({"isLoading": true}), rows));
        assert!(Arc::ptr_eq(&before.home_rows()[0], &after.home_rows()[0]));
        assert!(!Arc::ptr_eq(&before.home_rows()[1], &after.home_rows()[1]));
        assert!(!before.slice_shared(&after, "home"));
        assert!(before.slice_shared(&after, "navigation"));
        assert!(!before.slice_equal(&after, "home"));
    }

    #[test]
    fn equal_content_in_a_new_slice_counts_as_equal() {
        let a = sample();
        let b = sample();
        assert!(!a.slice_shared(&b, "library"));
        assert!(a.slice_equal(&b, "library"));
        assert!(a.slice_equal(&b, "home"));
    }
}
