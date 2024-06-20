use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct RipgrepOutput {
  #[serde(rename = "type")]
  pub kind: String,
  pub data: Option<RipgrepData>,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepData {
  pub path: Option<RipgrepPath>,
  pub lines: Option<RipgrepLines>,
  #[serde(rename = "line_number")]
  pub line_number: Option<u32>,
  #[serde(rename = "absolute_offset")]
  pub absolute_offset: Option<u32>,
  pub submatches: Option<Vec<RipgrepSubmatch>>,
  pub binary_offset: Option<Option<u64>>, // Note: binary_offset can be null
  pub stats: Option<RipgrepStats>,
  pub elapsed_total: Option<RipgrepElapsedTotal>,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepPath {
  pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct RipgrepLines {
  pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepSubmatch {
  pub r#match: RipgrepMatch,
  pub start: u32,
  pub end: u32,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepMatch {
  pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepElapsedTotal {
  pub human: String,
  pub nanos: u64,
  pub secs: u64,
}

#[derive(Deserialize, Debug)]
pub struct RipgrepStats {
  pub bytes_printed: u64,
  pub bytes_searched: u64,
  pub elapsed: RipgrepElapsedTotal,
  pub matched_lines: usize,
  pub matches: usize,
  pub searches: usize,
  pub searches_with_match: usize,
}

pub struct RipgrepSummary {
  pub elapsed_time: u64,
  pub matched_lines: usize,
  pub matches: usize,
  pub searches: usize,
  pub searches_with_match: usize,
}
