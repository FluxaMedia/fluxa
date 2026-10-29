#![allow(clippy::indexing_slicing)]

pub mod expr;
pub mod fetch;
pub mod format;
pub mod fuzzy;
pub mod groups;
pub mod join;
pub mod parse;
pub mod probe;
#[cfg(feature = "cli-probe")]
pub mod probe_cli;
pub mod release;
pub mod rules;
pub mod tracks;
pub mod types;
pub mod vocab;
