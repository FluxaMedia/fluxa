pub const NODE_DISCOVER_RECENT_BASE: u64 = 21_000;
const RECENT_NODE_SPAN: u64 = 32;

pub fn discover_recent(node: u64) -> Option<usize> {
    let slot = node.checked_sub(NODE_DISCOVER_RECENT_BASE)?;
    (slot < RECENT_NODE_SPAN).then_some(slot as usize)
}
