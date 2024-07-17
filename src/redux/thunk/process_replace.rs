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
    utils::{get_search_regex, replace_file_ast, replace_file_normal},
  },
  utils::is_git_repo,
};

pub struct ProcessReplaceThunk {
  command_tx: Arc<UnboundedSender<AppAction>>,
  force_replace: ForceReplace,
}

impl ProcessReplaceThunk {
  pub fn new(command_tx: Arc<UnboundedSender<AppAction>>, force_replace: ForceReplace) -> Self {
    Self { command_tx, force_replace }
  }

  async fn process_ast_grep_replace(&self, store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    for search_result in &search_list.list {
      replace_file_ast(search_result, &search_text_state, &replace_text_state);
    }
  }

  async fn process_normal_replace(&self, store: &Arc<impl StoreApi<State, Action> + Send + Sync + 'static>) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    let processing_status_action = AppAction::Tui(TuiAction::Status("Processing search and replace..".to_string()));
    self.command_tx.send(processing_status_action).unwrap();

    let re = get_search_regex(&search_text_state.text, &search_text_state.kind);

    for search_result in &search_list.list {
      replace_file_normal(search_result, &search_text_state, &replace_text_state);
    }
  }

  async fn handle_confirm<Api: StoreApi<State, Action> + Send + Sync + 'static>(&self, store: Arc<Api>) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    #[cfg(feature = "ast_grep")]
    if search_text_state.kind == SearchTextKind::AstGrep {
      self.process_ast_grep_replace(&store).await;
    } else {
      self.process_normal_replace(&store).await;
    }

    #[cfg(not(feature = "ast_grep"))]
    self.process_normal_replace(&store).await;

    store.dispatch(Action::ResetState).await;
    let reset_action = AppAction::Tui(TuiAction::Reset);
    self.command_tx.send(reset_action).unwrap();
    let done_processing_status_action = AppAction::Tui(TuiAction::Status("".to_string()));
    self.command_tx.send(done_processing_status_action).unwrap();

    let search_text_action = AppAction::Tui(TuiAction::Notify(NotificationEnum::Info(
      "Search and replace completed successfully".to_string(),
    )));
    self.command_tx.send(search_text_action).unwrap();
  }

  async fn handle_cancel(&self, store: Arc<impl StoreApi<State, Action>>) {
    let reset_action = Action::ResetState;
    store.dispatch(reset_action).await;
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for ProcessReplaceThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let project_root = store.select(|state: &State| state.project_root.clone()).await;
    let force_replace = self.force_replace.0;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    if force_replace {
      self.handle_confirm(store).await;
    } else if search_text_state.text.is_empty() {
      let search_text_action =
        AppAction::Tui(TuiAction::Notify(NotificationEnum::Error("Search text cannot be empty".to_string())));
      self.command_tx.send(search_text_action).unwrap();

      return;
    } else if replace_text_state.text.is_empty() {
      let confirm_dialog = Action::SetDialog {
        dialog: Some(Dialog::ConfirmReplace(ConfirmDialogState {
          message: "Replace text is empty, and replacing with an empty string will remove the matched text.\n Are you sure you want to continue?"
            .to_string(),
          on_confirm: Some(DialogAction::ConfirmReplace),
          on_cancel: Some(DialogAction::CancelReplace),
          confirm_label: "Continue".to_string(),
          cancel_label: "Cancel".to_string(),
          show_cancel: true,
          show: true,
        })),
      };

      store.dispatch(confirm_dialog).await;

      return;
    } else if is_git_repo(project_root) {
      self.handle_confirm(store.clone()).await;
    } else {
      let confirm_dialog = Action::SetDialog {
        dialog: Some(Dialog::ConfirmGitDirectory(ConfirmDialogState {
          message: "This action will modify the files in this directory.\n Are you sure you want to continue?"
            .to_string(),
          on_confirm: Some(DialogAction::ConfirmReplace),
          on_cancel: Some(DialogAction::CancelReplace),
          confirm_label: "Continue".to_string(),
          cancel_label: "Cancel".to_string(),
          show_cancel: true,
          show: true,
        })),
      };

      store.dispatch(confirm_dialog).await;

      return;
    }
  }
}
