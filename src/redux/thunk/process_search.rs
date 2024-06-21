use std::{
  collections::{HashMap, HashSet},
  process::Command,
  sync::Arc,
};

use async_trait::async_trait;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};

use crate::{
  redux::{
    action::Action,
    state::{Match, Metadata, SearchListState, SearchResultState, SearchTextKind, State, SubMatch},
  },
  ripgrep::{RipgrepOutput, RipgrepSummary},
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
    log::info!("Project root: {:?}", project_root);

    // Ensure search_text is set
    if !search_text_state.text.is_empty() {
      store.dispatch(Action::SetSearchList { search_list: SearchListState::default() }).await;

      let mut rg_args = vec!["--json"];

      // Determine the appropriate ripgrep command arguments based on the search kind
      match search_text_state.kind {
        SearchTextKind::Regex => rg_args.push(&search_text_state.text),
        SearchTextKind::MatchCase => rg_args.extend(&["-s", &search_text_state.text]),
        SearchTextKind::MatchWholeWord => rg_args.extend(&["-w", "-i", &search_text_state.text]),
        SearchTextKind::MatchCaseWholeWord => rg_args.extend(&["-w", "-s", &search_text_state.text]),
        SearchTextKind::Simple => rg_args.extend(&["-i", &search_text_state.text]),
      }

      let project_root_str = project_root.to_string_lossy();
      rg_args.push(&project_root_str);

      // log args
      log::info!("Ripgrep args: {:?}", rg_args);
      let output = Command::new("rg").args(&rg_args).output().expect("Failed to execute ripgrep");

      let stdout = String::from_utf8_lossy(&output.stdout);

      let mut results = Vec::new();
      let mut path_to_result: HashMap<String, usize> = HashMap::new();
      let mut summary: Option<RipgrepSummary> = None;

      for line in stdout.lines() {
        if let Ok(rg_output) = serde_json::from_str::<RipgrepOutput>(line) {
          match rg_output.kind.as_str() {
            "match" => {
              if let Some(data) = rg_output.data {
                let path = data.path.unwrap().text;
                let line_number = data.line_number.unwrap_or_default();
                let absolute_offset = data.absolute_offset.unwrap_or_default();

                let submatches: Vec<SubMatch> = data
                  .submatches
                  .unwrap_or_default()
                  .into_iter()
                  .map(|sm| SubMatch { start: sm.start as usize, end: sm.end as usize })
                  .collect();

                let mat = Match {
                  lines: data.lines,
                  line_number: line_number as usize,
                  absolute_offset: absolute_offset as usize,
                  submatches: submatches.clone(),
                };

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

                results[*search_result_index].matches.push(mat);
                // Increment the total_matches count by the number of submatches
                results[*search_result_index].total_matches += submatches.len();
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
