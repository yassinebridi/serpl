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
  let lines: Vec<&str> = content.lines().collect();

  let new_content = if replace_text_state.kind == ReplaceTextKind::DeleteLine {
    let matched_lines: std::collections::HashSet<usize> =
      search_result.matches.iter().map(|m| m.line_number - 1).collect();

    lines
      .iter()
      .enumerate()
      .filter(|(i, _)| !matched_lines.contains(i))
      .map(|(_, line)| *line)
      .collect::<Vec<&str>>()
      .join("\n")
  } else {
    let re = get_search_regex(&search_text_state.text, &search_text_state.kind);

    re.replace_all(&content, |caps: &regex::Captures| {
      apply_replace_captures(caps, &replace_text_state.text, &replace_text_state.kind, &search_text_state.kind)
    })
    .to_string()
  };

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
      RegexBuilder::new(&format!(r"\b{escaped_search_text}\b")).case_insensitive(true).build().expect("Invalid regex")
    },
    SearchTextKind::MatchCaseWholeWord => {
      RegexBuilder::new(&format!(r"\b{escaped_search_text}\b")).case_insensitive(false).build().expect("Invalid regex")
    },
    SearchTextKind::Regex => {
      RegexBuilder::new(search_text)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|_| RegexBuilder::new(r"(a)^").build().unwrap())
    },
    #[cfg(feature = "ast_grep")]
    SearchTextKind::AstGrep => unreachable!("AST Grep doesn't use regex"),
  }
}

/// Like `apply_replace`, but in regex search mode expands capture group references
/// (`$1`, `${name}`, `$$` for a literal `$`) in the replacement text first.
pub fn apply_replace_captures(
  caps: &regex::Captures,
  replace_text: &str,
  replace_kind: &ReplaceTextKind,
  search_kind: &SearchTextKind,
) -> String {
  let matched_text = caps.get(0).unwrap().as_str();
  if *search_kind == SearchTextKind::Regex && *replace_kind != ReplaceTextKind::DeleteLine {
    let mut expanded = String::new();
    caps.expand(replace_text, &mut expanded);
    apply_replace(matched_text, &expanded, replace_kind)
  } else {
    apply_replace(matched_text, replace_text, replace_kind)
  }
}

pub fn apply_replace(matched_text: &str, replace_text: &str, replace_kind: &ReplaceTextKind) -> String {
  match replace_kind {
    ReplaceTextKind::Simple => replace_text.to_string(),
    ReplaceTextKind::DeleteLine => String::new(),
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

#[cfg(test)]
mod tests {
  use super::*;

  fn replace(search: &str, replace: &str, kind: SearchTextKind, text: &str) -> String {
    let re = get_search_regex(search, &kind);
    re.replace_all(text, |caps: &regex::Captures| {
      apply_replace_captures(caps, replace, &ReplaceTextKind::Simple, &kind)
    })
    .to_string()
  }

  #[test]
  fn regex_capture_groups_are_expanded() {
    assert_eq!(replace(r"(.*)fun.*(\d+)", "\"$2$1$2\"", SearchTextKind::Regex, "my_function2()"), "\"2my_2\"()");
  }

  #[test]
  fn named_groups_are_expanded() {
    assert_eq!(replace(r"(?P<k>\w+)=(?P<v>\w+)", "${v}=${k}", SearchTextKind::Regex, "a=b c=d"), "b=a d=c");
  }

  #[test]
  fn escaped_dollar_is_literal() {
    assert_eq!(replace(r"(\d+)", "$$$1", SearchTextKind::Regex, "cost 5"), "cost $5");
  }

  #[test]
  fn missing_group_expands_to_empty() {
    assert_eq!(replace(r"(a)", "[$2]", SearchTextKind::Regex, "a"), "[]");
  }

  #[test]
  fn delete_line_ignores_captures() {
    let kind = SearchTextKind::Regex;
    let re = get_search_regex("(a)", &kind);
    let caps = re.captures("a").unwrap();
    assert_eq!(apply_replace_captures(&caps, "$1", &ReplaceTextKind::DeleteLine, &kind), "");
  }

  #[test]
  fn preserve_case_applies_after_expansion() {
    let kind = SearchTextKind::Regex;
    let re = get_search_regex("(hello)", &kind);
    let caps = re.captures("HELLO").unwrap();
    assert_eq!(apply_replace_captures(&caps, "bye-$1", &ReplaceTextKind::PreserveCase, &kind), "BYE-HELLO");
  }

  #[test]
  fn non_regex_search_keeps_dollar_literal() {
    assert_eq!(replace("a", "$1", SearchTextKind::Simple, "a"), "$1");
  }
}
