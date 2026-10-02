use std::{
  collections::{HashMap, HashSet, VecDeque},
  fs,
  path::PathBuf,
  process::Command,
  sync::Arc,
};

use async_trait::async_trait;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};
use serde_json::from_str;

use crate::{
  astgrep::AstGrepOutput,
  redux::{
    action::Action,
    state::{Match, Metadata, SearchListState, SearchResultState, SearchTextKind, SearchTextState, State, SubMatch},
  },
  ripgrep::{RipgrepLines, RipgrepOutput, RipgrepSummary},
};

/// Builds the ripgrep arguments for a search.
///
/// When `include_hidden` is set, hidden files are searched too, but `.git` is always
/// excluded so replacing never touches repository internals.
pub fn build_rg_args(search_text_state: &SearchTextState, project_root: &str, include_hidden: bool) -> Vec<String> {
  let mut rg_args: Vec<String> = vec!["--json".into(), "-C".into(), "3".into()];

  if include_hidden {
    rg_args.extend(["--hidden".into(), "--glob".into(), "!.git".into()]);
  }

  let text = search_text_state.text.clone();
  match search_text_state.kind {
    SearchTextKind::Regex => rg_args.push(text),
    SearchTextKind::MatchCase => rg_args.extend(["-s".into(), text]),
    SearchTextKind::MatchWholeWord => rg_args.extend(["-w".into(), "-i".into(), text]),
    SearchTextKind::MatchCaseWholeWord => rg_args.extend(["-w".into(), "-s".into(), text]),
    SearchTextKind::Simple => rg_args.extend(["-i".into(), "-F".into(), text]),
    #[cfg(feature = "ast_grep")]
    SearchTextKind::AstGrep => {},
  }

  rg_args.push(project_root.to_string());
  rg_args
}

pub struct ProcessSearchThunk {}

impl ProcessSearchThunk {
  pub fn new() -> Self {
    Self {}
  }

  fn get_context(lines: &[&str], start: usize, count: usize, forward: bool) -> Vec<String> {
    let mut context = Vec::new();
    let mut current = start;

    for _ in 0..count {
      if forward {
        if current >= lines.len() {
          break;
        }
        context.push(lines[current].to_string());
        current += 1;
      } else {
        if current == 0 {
          break;
        }
        current -= 1;
        context.insert(0, lines[current].to_string());
      }
    }

    context
  }

  async fn process_ast_grep_search(&self, store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>) {
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;
    let replace_text = replace_text_state.text.clone();
    let project_root = store.select(|state: &State| state.project_root.clone()).await;
    let include_hidden = store.select(|state: &State| state.include_hidden).await;

    let mut args = vec!["run", "-p", &search_text_state.text, "--json=compact", project_root.to_str().unwrap()];
    if include_hidden {
      args.extend(["--no-ignore", "hidden", "--globs", "!.git"]);
    }
    if !replace_text.is_empty() {
      args.push("-r");
      args.push(&replace_text);
    }
    let output = Command::new("ast-grep").args(args).output().expect("Failed to execute ast-grep");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let ast_grep_results: Vec<AstGrepOutput> = from_str(&stdout).expect("Failed to parse ast-grep output");
    let mut aggregated_results: HashMap<String, SearchResultState> = HashMap::new();
    for result in ast_grep_results {
      let file_content = fs::read_to_string(&result.file).unwrap_or_default();
      let lines: Vec<&str> = file_content.lines().collect();

      let context_before = Self::get_context(&lines, result.range.start.line, 3, false);
      let context_after = Self::get_context(&lines, result.range.end.line, 3, true);

      aggregated_results
        .entry(result.file.clone())
        .or_insert_with(|| {
          SearchResultState { index: None, path: result.file.clone(), matches: Vec::new(), total_matches: 0 }
        })
        .matches
        .push(Match {
          line_number: result.range.start.line,
          lines: Some(RipgrepLines { text: result.lines }),
          absolute_offset: result.range.byte_offset.start,
          submatches: vec![SubMatch {
            start: result.range.start.column,
            end: result.range.end.column,
            line_start: result.range.start.line,
            line_end: result.range.end.line,
          }],
          replacement: result.replacement,
          context_before,
          context_after,
        });
    }

    let mut search_results: Vec<SearchResultState> = aggregated_results.into_values().collect();
    for (index, result) in search_results.iter_mut().enumerate() {
      result.index = Some(index);
      result.total_matches = result.matches.len();
    }

    let search_list_state = SearchListState {
      list: search_results.clone(),
      metadata: Metadata {
        elapsed_time: 0,
        matched_lines: search_results.iter().map(|r| r.total_matches).sum(),
        matches: search_results.iter().map(|r| r.total_matches).sum(),
        searches: 1,
        searches_with_match: if search_results.is_empty() { 0 } else { 1 },
      },
    };

    store.dispatch(Action::SetSearchList { search_list: search_list_state }).await;
  }

  async fn process_normal_search(&self, store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>) {
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let project_root = store.select(|state: &State| state.project_root.clone()).await;
    let include_hidden = store.select(|state: &State| state.include_hidden).await;
    let rg_args = build_rg_args(&search_text_state, &project_root.to_string_lossy(), include_hidden);

    let output = Command::new("rg").args(&rg_args).output().expect("Failed to execute ripgrep");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut results = Vec::new();
    let mut path_to_result: HashMap<String, usize> = HashMap::new();
    let mut summary: Option<RipgrepSummary> = None;

    let mut context_buffer: VecDeque<(usize, String)> = VecDeque::new();

    for line in stdout.lines() {
      if let Ok(rg_output) = serde_json::from_str::<RipgrepOutput>(line) {
        match rg_output.kind.as_str() {
          "match" | "context" => {
            if let Some(data) = rg_output.data {
              let path = data.path.unwrap().text;
              let line_number = data.line_number.unwrap_or_default() as usize;
              let absolute_offset = data.absolute_offset.unwrap_or_default();

              let search_result_index = path_to_result.entry(path.clone()).or_insert_with(|| {
                let index = results.len();
                results.push(SearchResultState {
                  index: Some(index),
                  path: path.clone(),
                  matches: Vec::new(),
                  total_matches: 0,
                });
                index
              });

              let result = &mut results[*search_result_index];

              if rg_output.kind == "match" {
                let submatches: Vec<SubMatch> = data
                  .submatches
                  .unwrap_or_default()
                  .into_iter()
                  .map(|sm| SubMatch { start: sm.start as usize, end: sm.end as usize, line_start: 0, line_end: 0 })
                  .collect();

                let mut context_before: Vec<String> = context_buffer.drain(..).map(|(_, line)| line).collect();
                if context_before.len() > 3 {
                  context_before = context_before.clone().into_iter().skip(context_before.len() - 3).collect();
                }

                result.matches.push(Match {
                  lines: data.lines.clone(),
                  line_number,
                  context_before,
                  context_after: Vec::new(),
                  absolute_offset: absolute_offset as usize,
                  submatches: submatches.clone(),
                  replacement: None,
                });
                result.total_matches += submatches.len();

                context_buffer.push_back((line_number, data.lines.unwrap().text));
              } else {
                context_buffer.push_back((line_number, data.lines.clone().unwrap().text));
                if context_buffer.len() > 4 {
                  context_buffer.pop_front();
                }

                if let Some(last_match) = result.matches.last_mut() {
                  if line_number > last_match.line_number && last_match.context_after.len() < 3 {
                    last_match.context_after.push(data.lines.unwrap().text);
                  }
                }
              }
            }
          },
          "summary" => {
            if let Some(data) = rg_output.data {
              summary = Some(RipgrepSummary {
                elapsed_time: data.elapsed_total.unwrap().nanos,
                matched_lines: data.stats.as_ref().unwrap().matched_lines,
                matches: data.stats.as_ref().unwrap().matches,
                searches: data.stats.as_ref().unwrap().searches,
                searches_with_match: data.stats.as_ref().unwrap().searches_with_match,
              });
            }
          },
          _ => {},
        }
      }
    }

    let metadata = if let Some(s) = summary {
      Metadata {
        elapsed_time: s.elapsed_time,
        matched_lines: s.matched_lines,
        matches: s.matches,
        searches: s.searches,
        searches_with_match: s.searches_with_match,
      }
    } else {
      Metadata::default()
    };

    let search_list_state = SearchListState { list: results, metadata };

    store.dispatch(Action::SetSearchList { search_list: search_list_state }).await;
  }
}

impl Default for ProcessSearchThunk {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for ProcessSearchThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;

    if !search_text_state.text.is_empty() {
      store.dispatch(Action::SetSearchList { search_list: SearchListState::default() }).await;

      #[cfg(feature = "ast_grep")]
      if search_text_state.kind == SearchTextKind::AstGrep {
        self.process_ast_grep_search(&store).await;
      } else {
        self.process_normal_search(&store).await;
      }
      #[cfg(not(feature = "ast_grep"))]
      self.process_normal_search(&store).await;
    }
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  fn state(kind: SearchTextKind) -> SearchTextState {
    SearchTextState { text: "foo".to_string(), kind }
  }

  #[test]
  fn rg_args_exclude_hidden_by_default() {
    let args = build_rg_args(&state(SearchTextKind::Simple), "/root", false);
    assert_eq!(args, vec!["--json", "-C", "3", "-i", "-F", "foo", "/root"]);
  }

  #[test]
  fn rg_args_include_hidden_but_skip_git() {
    let args = build_rg_args(&state(SearchTextKind::Simple), "/root", true);
    assert_eq!(args, vec!["--json", "-C", "3", "--hidden", "--glob", "!.git", "-i", "-F", "foo", "/root"]);
  }

  #[test]
  fn rg_args_hidden_flags_precede_pattern_for_every_kind() {
    for kind in [
      SearchTextKind::Regex,
      SearchTextKind::MatchCase,
      SearchTextKind::MatchWholeWord,
      SearchTextKind::MatchCaseWholeWord,
      SearchTextKind::Simple,
    ] {
      let args = build_rg_args(&state(kind), "/root", true);
      let hidden = args.iter().position(|a| a == "--hidden").unwrap();
      let pattern = args.iter().position(|a| a == "foo").unwrap();
      assert!(hidden < pattern, "{kind:?}: {args:?}");
      assert_eq!(args.last().unwrap(), "/root");
    }
  }
}
