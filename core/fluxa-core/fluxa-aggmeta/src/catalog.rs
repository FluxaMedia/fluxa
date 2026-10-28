use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ProviderPage<T> {
    pub items: Vec<T>,
    pub has_next_page: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub page: u32,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Filled<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

#[derive(Default)]
pub struct CursorStore {
    cursors: HashMap<(String, String, usize), Cursor>,
}

impl CursorStore {
    pub fn start_for(&self, catalog: &str, filters: &str, skip: usize, page_size: usize) -> Cursor {
        self.cursors
            .get(&(catalog.to_string(), filters.to_string(), skip))
            .copied()
            .unwrap_or(Cursor {
                page: (skip / page_size.max(1)) as u32 + 1,
                offset: skip % page_size.max(1),
            })
    }

    fn remember(&mut self, catalog: &str, filters: &str, skip: usize, cursor: Cursor) {
        self.cursors
            .insert((catalog.to_string(), filters.to_string(), skip), cursor);
    }

    pub fn fill<T, E>(
        &mut self,
        catalog: &str,
        filters: &str,
        skip: usize,
        limit: usize,
        page_size: usize,
        keep: impl Fn(&T) -> bool,
        mut fetch: impl FnMut(u32) -> Result<ProviderPage<T>, E>,
    ) -> Result<Filled<T>, E> {
        let mut cursor = self.start_for(catalog, filters, skip, page_size);
        let mut out: Vec<T> = Vec::with_capacity(limit);
        loop {
            let page = fetch(cursor.page)?;
            let total = page.items.len();
            let has_next = page.has_next_page;
            let mut consumed = cursor.offset.min(total);
            for item in page.items.into_iter().skip(cursor.offset) {
                consumed += 1;
                if keep(&item) {
                    out.push(item);
                    if out.len() == limit {
                        break;
                    }
                }
            }
            let leftover_here = consumed < total;
            if out.len() == limit {
                let next = if leftover_here {
                    Cursor {
                        page: cursor.page,
                        offset: consumed,
                    }
                } else {
                    Cursor {
                        page: cursor.page + 1,
                        offset: 0,
                    }
                };
                self.remember(catalog, filters, skip + out.len(), next);
                return Ok(Filled {
                    items: out,
                    has_more: leftover_here || has_next,
                });
            }
            if !has_next {
                self.remember(
                    catalog,
                    filters,
                    skip + out.len(),
                    Cursor {
                        page: cursor.page + 1,
                        offset: 0,
                    },
                );
                return Ok(Filled {
                    items: out,
                    has_more: false,
                });
            }
            cursor = Cursor {
                page: cursor.page + 1,
                offset: 0,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(data: Vec<Vec<u32>>) -> impl FnMut(u32) -> Result<ProviderPage<u32>, ()> {
        move |p| {
            let idx = p as usize - 1;
            Ok(ProviderPage {
                items: data.get(idx).cloned().unwrap_or_default(),
                has_next_page: idx + 1 < data.len(),
            })
        }
    }

    #[test]
    fn filter_fill_spans_provider_pages() {
        let mut store = CursorStore::default();
        let data = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        let got = store
            .fill("c", "even", 0, 5, 4, |n| n % 2 == 0, pages(data))
            .unwrap();
        assert_eq!(got.items, vec![2, 4, 6, 8, 10]);
        assert!(got.has_more);
    }

    #[test]
    fn next_skip_resumes_without_refetching_earlier_pages() {
        let mut store = CursorStore::default();
        let data = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];
        store
            .fill("c", "even", 0, 3, 4, |n| n % 2 == 0, pages(data.clone()))
            .unwrap();
        let mut fetched = Vec::new();
        let mut inner = pages(data);
        let got = store
            .fill(
                "c",
                "even",
                3,
                3,
                4,
                |n| n % 2 == 0,
                |p| {
                    fetched.push(p);
                    inner(p)
                },
            )
            .unwrap();
        assert_eq!(got.items, vec![8, 10, 12]);
        assert_eq!(fetched.first(), Some(&2));
    }

    #[test]
    fn exhausted_provider_reports_no_more() {
        let mut store = CursorStore::default();
        let data = vec![vec![1, 2], vec![3]];
        let got = store
            .fill("c", "", 0, 10, 2, |_| true, pages(data))
            .unwrap();
        assert_eq!(got.items, vec![1, 2, 3]);
        assert!(!got.has_more);
    }

    #[test]
    fn leftovers_on_the_last_page_keep_has_more_true() {
        let mut store = CursorStore::default();
        let got = store
            .fill("c", "", 0, 2, 4, |_| true, pages(vec![vec![1, 2, 3, 4]]))
            .unwrap();
        assert_eq!(got.items, vec![1, 2]);
        assert!(got.has_more);
    }

    #[test]
    fn unfiltered_skip_maps_to_page_and_offset() {
        let store = CursorStore::default();
        assert_eq!(
            store.start_for("c", "", 25, 20),
            Cursor { page: 2, offset: 5 }
        );
    }
}
