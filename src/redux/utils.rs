use std::fs;

use regex::RegexBuilder;
use serde_json::from_str;

use crate::{
  astgrep::AstGrepOutput,
  redux::state::{ReplaceTextKind, SearchTextKind},
};

pub fn replace_file_ast(
  search_result: &crate::redux::state::SearchResultState,
  search_text_state: &crate::redux::state::SearchTextState,
  replace_text_state: &crate::redux::state::ReplaceTextState,
) {
  let file_path = &search_result.path;

  let mut content = fs::read_to_string(file_path).expect("Unable to read file");
  let lines: Vec<&str> = content.lines().collect();

  let lines_to_replace: std::collections::HashSet<usize> =
    search_result.matches.iter().map(|m| m.line_number).collect();

  let output = std::process::Command::new("ast-grep")
    .args(["run", "-p", &search_text_state.text, "-r", &replace_text_state.text, "--json=compact", file_path])
    .output()
    .expect("Failed to execute ast-grep for replacement");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let ast_grep_results: Vec<AstGrepOutput> = from_str(&stdout).expect("Failed to parse ast-grep output");

  for result in ast_grep_results.iter().rev() {
    if let (Some(replacement), Some(offsets)) = (&result.replacement, &result.replacement_offsets) {
      if lines_to_replace.contains(&result.range.start.line) {
        let start = offsets.start;
        let end = offsets.end;
        content.replace_range(start..end, replacement);
      }
    }
  }

  fs::write(file_path, content).expect("Unable to write file");
}

pub fn replace_file_normal(
  search_result: &crate::redux::state::SearchResultState,
  search_text_state: &crate::redux::state::SearchTextState,
  replace_text_state: &crate::redux::state::ReplaceTextState,
) {
  let file_path = &search_result.path;

  let content = fs::read_to_string(file_path).expect("Unable to read file");

  let re = get_search_regex(&search_text_state.text, &search_text_state.kind);

  let new_content = re
    .replace_all(&content, |caps: &regex::Captures| {
      let matched_text = caps.get(0).unwrap().as_str();
      apply_replace(matched_text, &replace_text_state.text, &replace_text_state.kind)
    })
    .to_string();

  fs::write(file_path, new_content).expect("Unable to write file");
}

pub fn get_search_regex(search_text: &str, search_kind: &SearchTextKind) -> regex::Regex {
  let escaped_search_text = regex::escape(search_text);

  match search_kind {
    SearchTextKind::Simple => {
      RegexBuilder::new(&escaped_search_text).case_insensitive(true).build().expect("Invalid regex")
    },
    SearchTextKind::MatchCase => {
      RegexBuilder::new(&escaped_search_text).case_insensitive(false).build().expect("Invalid regex")
    },
    SearchTextKind::MatchWholeWord => {
      RegexBuilder::new(&format!(r"\b{}\b", escaped_search_text)).case_insensitive(true).build().expect("Invalid regex")
    },
    SearchTextKind::MatchCaseWholeWord => {
      RegexBuilder::new(&format!(r"\b{}\b", escaped_search_text))
        .case_insensitive(false)
        .build()
        .expect("Invalid regex")
    },
    SearchTextKind::Regex => {
      RegexBuilder::new(&escaped_search_text).case_insensitive(true).build().expect("Invalid regex")
    },
    #[cfg(feature = "ast_grep")]
    SearchTextKind::AstGrep => unreachable!("AST Grep doesn't use regex"),
  }
}

pub fn apply_replace(matched_text: &str, replace_text: &str, replace_kind: &ReplaceTextKind) -> String {
  match replace_kind {
    ReplaceTextKind::Simple => replace_text.to_string(),
    ReplaceTextKind::PreserveCase => {
      let first_char = matched_text.chars().next().unwrap_or_default();
      if matched_text.chars().all(char::is_uppercase) {
        replace_text.to_uppercase()
      } else if first_char.is_uppercase() {
        let mut result = String::new();
        for (i, c) in replace_text.chars().enumerate() {
          if i == 0 {
            result.push(c.to_uppercase().next().unwrap());
          } else {
            result.push(c.to_lowercase().next().unwrap());
          }
        }
        result
      } else {
        replace_text.to_lowercase()
      }
    },
    #[cfg(feature = "ast_grep")]
    ReplaceTextKind::AstGrep => unreachable!(),
  }
}
