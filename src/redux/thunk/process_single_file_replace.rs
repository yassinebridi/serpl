use std::{collections::HashSet, fs, io::Write, path::PathBuf, process::Command, sync::Arc, time::Duration};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::style::Color;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};
use regex::RegexBuilder;
use serde_json::from_str;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::{AppAction, TuiAction},
  astgrep::AstGrepOutput,
  components::notifications::NotificationEnum,
  redux::{
    action::Action,
    state::{ConfirmDialogState, Dialog, DialogAction, ReplaceTextKind, SearchTextKind, State},
    thunk::{ForceReplace, ThunkAction},
    utils::{replace_file_ast, replace_file_normal},
  },
  utils::is_git_repo,
};

pub struct ProcessSingleFileReplaceThunk {
  command_tx: Arc<UnboundedSender<AppAction>>,
  /// Indices into the result list, in descending order so removing one keeps the rest valid.
  file_indices: Vec<usize>,
}

impl ProcessSingleFileReplaceThunk {
  pub fn new(command_tx: Arc<UnboundedSender<AppAction>>, file_index: usize) -> Self {
    Self { command_tx, file_indices: vec![file_index] }
  }

  /// Replaces several files one after another in a single thunk. Thunks are spawned concurrently, so
  /// dispatching one per file would race on the shifting list indices.
  pub fn new_many(command_tx: Arc<UnboundedSender<AppAction>>, mut file_indices: Vec<usize>) -> Self {
    file_indices.sort_unstable_by(|a, b| b.cmp(a));
    file_indices.dedup();
    Self { command_tx, file_indices }
  }

  async fn process_ast_grep_replace(
    &self,
    store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>,
    file_index: usize,
  ) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    if let Some(search_result) = search_list.list.get(file_index) {
      replace_file_ast(search_result, &search_text_state, &replace_text_state);
    }
  }

  async fn process_normal_replace(
    &self,
    store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>,
    file_index: usize,
  ) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    if let Some(search_result) = search_list.list.get(file_index) {
      let file_path = &search_result.path;
      let content = fs::read_to_string(file_path).expect("Unable to read file");
      let mut lines: Vec<String> = content.lines().map(String::from).collect();

      if replace_text_state.kind == ReplaceTextKind::DeleteLine {
        let matched_lines: Vec<usize> = search_result.matches.iter().map(|m| m.line_number - 1).collect();
        for &line_index in matched_lines.iter().rev() {
          if line_index < lines.len() {
            lines.remove(line_index);
          }
        }
        let new_content = lines.join("\n");
        fs::write(file_path, new_content).expect("Unable to write file");
      } else {
        replace_file_normal(search_result, &search_text_state, &replace_text_state);
      }
    }
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for ProcessSingleFileReplaceThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    for &file_index in &self.file_indices {
      #[cfg(feature = "ast_grep")]
      if search_text_state.kind == SearchTextKind::AstGrep {
        self.process_ast_grep_replace(&store, file_index).await;
      } else {
        self.process_normal_replace(&store, file_index).await;
      }

      #[cfg(not(feature = "ast_grep"))]
      self.process_normal_replace(&store, file_index).await;

      store.dispatch(Action::RemoveFileFromList { index: file_index }).await;
    }

    let done_processing_status_action = AppAction::Tui(TuiAction::Status("".to_string()));
    self.command_tx.send(done_processing_status_action).unwrap();

    let notification_action =
      AppAction::Tui(TuiAction::Notify(NotificationEnum::Info("File replacement completed successfully".to_string())));
    self.command_tx.send(notification_action).unwrap();
  }
}

#[cfg(test)]
mod tests {
  use redux_rs::Store;

  use super::*;
  use crate::redux::{reducer::reducer, state::SearchResultState};

  #[tokio::test]
  async fn replacing_every_file_empties_the_list() {
    let dir = std::env::temp_dir().join("serpl_multi_replace_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let mut state = State::default();
    for i in 0..4 {
      let path = dir.join(format!("f{i}.txt")).to_string_lossy().to_string();
      fs::write(&path, "foo\n").unwrap();
      state.search_result.list.push(SearchResultState { path, ..Default::default() });
    }
    state.search_text.text = "foo".into();
    state.replace_text.text = "bar".into();

    let store = Arc::new(Store::new_with_state(reducer, state));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = Arc::new(tx);
    let thunk = ProcessSingleFileReplaceThunk::new_many(tx, vec![2, 0, 3, 1]);
    thunk.execute(store.clone()).await;
    assert_eq!(fs::read_to_string(dir.join("f3.txt")).unwrap(), "bar\n");
    let left = store.select(|s: &State| s.search_result.list.len()).await;
    assert_eq!(left, 0);
  }
}
