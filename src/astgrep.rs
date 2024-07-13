use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AstGrepOutput {
  pub text: String,
  pub range: Range,
  pub file: String,
  pub lines: String,
  pub replacement: Option<String>,
  #[serde(rename = "replacementOffsets")]
  pub replacement_offsets: Option<ReplacementOffsets>,
  pub language: String,
  #[serde(rename = "metaVariables")]
  pub meta_variables: Option<MetaVariables>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Range {
  #[serde(rename = "byteOffset")]
  pub byte_offset: ByteOffset,
  pub start: Position,
  pub end: Position,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ByteOffset {
  pub start: usize,
  pub end: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Position {
  pub line: usize,
  pub column: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReplacementOffsets {
  pub start: usize,
  pub end: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetaVariables {
  pub single: HashMap<String, MetaVariable>,
  pub multi: HashMap<String, Vec<MetaVariable>>,
  pub transformed: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetaVariable {
  pub text: String,
  pub range: Range,
}
