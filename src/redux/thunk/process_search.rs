use std::{
  collections::{HashMap, HashSet, VecDeque},
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
    state::{Match, Metadata, SearchListState, SearchResultState, SearchTextKind, State, SubMatch},
  },
  ripgrep::{RipgrepLines, RipgrepOutput, RipgrepSummary},
};

pub struct ProcessSearchThunk {}

impl ProcessSearchThunk {
  pub fn new() -> Self {
    Self {}
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
    let project_root = store.select(|state: &State| state.project_root.clone()).await;

    if !search_text_state.text.is_empty() {
      store.dispatch(Action::SetSearchList { search_list: SearchListState::default() }).await;
      if search_text_state.kind == SearchTextKind::AstGrep {
        let output = Command::new("sg")
          .args(["run", "-p", &search_text_state.text, "--json=compact", project_root.to_str().unwrap()])
          .output()
          .expect("Failed to execute ast-grep");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ast_grep_results: Vec<AstGrepOutput> = from_str(&stdout).expect("Failed to parse ast-grep output");

        let search_results: Vec<SearchResultState> = ast_grep_results
          .into_iter()
          .map(|result| {
            SearchResultState {
              index: None, // You might want to set this based on some criteria
              path: result.file,
              matches: vec![Match {
                line_number: result.range.start.line,
                lines: Some(RipgrepLines { text: result.lines }),
                absolute_offset: result.range.byte_offset.start,
                submatches: vec![SubMatch { start: result.range.start.column, end: result.range.end.column }],
                context_before: Vec::new(),
                context_after: Vec::new(),
              }],
              total_matches: 1,
            }
          })
          .collect();

        let search_list_state = SearchListState {
          list: search_results.clone(),
          metadata: Metadata {
            elapsed_time: 0, // You might want to measure this
            matched_lines: search_results.clone().len(),
            matches: search_results.len(),
            searches: 1,
            searches_with_match: if search_results.is_empty() { 0 } else { 1 },
          },
        };

        store.dispatch(Action::SetSearchList { search_list: search_list_state }).await;
      } else {
        let mut rg_args = vec!["--json", "-C", "3"];

        match search_text_state.kind {
          SearchTextKind::Regex => rg_args.push(&search_text_state.text),
          SearchTextKind::MatchCase => rg_args.extend(&["-s", &search_text_state.text]),
          SearchTextKind::MatchWholeWord => rg_args.extend(&["-w", "-i", &search_text_state.text]),
          SearchTextKind::MatchCaseWholeWord => rg_args.extend(&["-w", "-s", &search_text_state.text]),
          SearchTextKind::Simple => rg_args.extend(&["-i", &search_text_state.text]),
          _ => {},
        }

        let project_root_str = project_root.to_string_lossy();
        rg_args.push(&project_root_str);

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
                      .map(|sm| SubMatch { start: sm.start as usize, end: sm.end as usize })
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
  }
}
