use std::{fs, process::Command, sync::Arc};

use async_trait::async_trait;
use redux_rs::{middlewares::thunk::Thunk, StoreApi};
use regex::RegexBuilder;
use serde_json::from_str;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::{AppAction, TuiAction},
  astgrep::AstGrepOutput,
  components::notifications::NotificationEnum,
  redux::{
    action::Action,
    state::{Match, ReplaceTextKind, ReplaceTextState, SearchTextKind, SearchTextState, State},
    thunk::ThunkAction,
    utils::{apply_replace, get_search_regex},
  },
};

pub struct ProcessLineReplaceThunk {
  command_tx: Arc<UnboundedSender<AppAction>>,
  file_index: usize,
  line_index: usize,
}

impl ProcessLineReplaceThunk {
  pub fn new(command_tx: Arc<UnboundedSender<AppAction>>, file_index: usize, line_index: usize) -> Self {
    Self { command_tx, file_index, line_index }
  }

  async fn process_replace_line(&self, store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    if let Some(search_result) = search_list.list.get(self.file_index) {
      if let Some(match_info) = search_result.matches.get(self.line_index) {
        let file_path = &search_result.path;

        #[cfg(feature = "ast_grep")]
        if search_text_state.kind == SearchTextKind::AstGrep {
          self
            .process_ast_grep_replace(
              file_path,
              &search_text_state.text,
              &replace_text_state.text,
              match_info.line_number,
            )
            .await;
        } else {
          process_normal_replace(search_text_state, match_info, replace_text_state, file_path);
        }

        #[cfg(not(feature = "ast_grep"))]
        process_normal_replace(search_text_state, match_info, replace_text_state, file_path);
      }
    }
  }

  async fn process_ast_grep_replace(
    &self,
    file_path: &str,
    search_pattern: &str,
    replace_pattern: &str,
    line_number: usize,
  ) {
    let output = Command::new("ast-grep")
      .args(["run", "-p", search_pattern, "-r", replace_pattern, "--json=compact", file_path])
      .output()
      .expect("Failed to execute ast-grep for replacement");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ast_grep_results: Vec<AstGrepOutput> = from_str(&stdout).expect("Failed to parse ast-grep output");

    let mut content = fs::read_to_string(file_path).expect("Unable to read file");

    for result in ast_grep_results.iter().rev() {
      if result.range.start.line == line_number {
        if let (Some(replacement), Some(offsets)) = (&result.replacement, &result.replacement_offsets) {
          let start = offsets.start;
          let end = offsets.end;
          content.replace_range(start..end, replacement);
        }
      }
    }

    fs::write(file_path, content).expect("Unable to write file");
  }
}

fn process_normal_replace(
  search_text_state: SearchTextState,
  match_info: &Match,
  replace_text_state: ReplaceTextState,
  file_path: &str,
) {
  let content = fs::read_to_string(file_path).expect("Unable to read file");
  let mut lines: Vec<String> = content.lines().map(String::from).collect();

  if replace_text_state.kind == ReplaceTextKind::DeleteLine {
    if match_info.line_number > 0 && match_info.line_number <= lines.len() {
      lines.remove(match_info.line_number - 1);
    }
  } else {
    let re = get_search_regex(&search_text_state.text, &search_text_state.kind);

    if let Some(line) = lines.get_mut(match_info.line_number - 1) {
      let replaced_line = re.replace_all(line, |caps: &regex::Captures| {
        let matched_text = caps.get(0).unwrap().as_str();
        apply_replace(matched_text, &replace_text_state.text, &replace_text_state.kind)
      });
      *line = replaced_line.into_owned();
    }
  }

  let new_content = lines.join("\n");
  fs::write(file_path, new_content).expect("Unable to write file");
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for ProcessLineReplaceThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let processing_status_action = AppAction::Tui(TuiAction::Status("Processing line replacement...".to_string()));
    self.command_tx.send(processing_status_action).unwrap();

    self.process_replace_line(&store).await;

    store.dispatch(Action::RemoveLineFromFile { file_index: self.file_index, line_index: self.line_index }).await;

    let done_processing_status_action = AppAction::Tui(TuiAction::Status("".to_string()));
    self.command_tx.send(done_processing_status_action).unwrap();

    let notification_action =
      AppAction::Tui(TuiAction::Notify(NotificationEnum::Info("Line replacement completed successfully".to_string())));
    self.command_tx.send(notification_action).unwrap();
  }
}
