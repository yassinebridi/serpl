use std::{fs, io::Write, path::PathBuf, sync::Arc, time::Duration};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::style::Color;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};
use regex::RegexBuilder;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::{AppAction, TuiAction},
  components::notifications::NotificationEnum,
  redux::{
    action::Action,
    state::{ConfirmDialog, Dialog, DialogAction, ReplaceTextKind, SearchTextKind, State},
    thunk::{ForceReplace, ThunkAction},
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

  async fn handle_confirm(&self, store: Arc<impl StoreApi<State, Action>>) {
    let search_list = store.select(|state: &State| state.search_result.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;

    let processing_status_action = AppAction::Tui(TuiAction::Status("Processing search and replace..".to_string()));
    self.command_tx.send(processing_status_action).unwrap();

    // Compile regex with case-insensitive flag if needed
    let re = match search_text_state.kind {
      SearchTextKind::Regex => {
        RegexBuilder::new(&search_text_state.text).case_insensitive(true).build().expect("Invalid regex")
      },
      SearchTextKind::MatchCase => {
        RegexBuilder::new(&regex::escape(&search_text_state.text))
          .case_insensitive(false)
          .build()
          .expect("Invalid regex")
      },
      SearchTextKind::MatchWholeWord => {
        RegexBuilder::new(&format!(r"\b{}\b", regex::escape(&search_text_state.text)))
          .case_insensitive(true)
          .build()
          .expect("Invalid regex")
      },
      SearchTextKind::MatchCaseWholeWord => {
        RegexBuilder::new(&format!(r"\b{}\b", regex::escape(&search_text_state.text)))
          .case_insensitive(false)
          .build()
          .expect("Invalid regex")
      },
      SearchTextKind::Simple => {
        RegexBuilder::new(&regex::escape(&search_text_state.text))
          .case_insensitive(true)
          .build()
          .expect("Invalid regex")
      },
    };

    for search_result in &search_list.list {
      let file_path = &search_result.path;

      // Read the file content
      let content = fs::read_to_string(file_path).expect("Unable to read file");

      // Create a new content buffer
      let mut new_content = String::new();

      // Track the last end index to handle replacements
      let mut last_end = 0;

      for mat in &search_result.matches {
        let line_number = mat.line_number;
        let line_start = content
                      .lines()
                      .take(line_number - 1)
                      .map(|line| line.len() + 1) // +1 for the newline character
                      .sum::<usize>();
        let line_end = line_start + mat.lines.as_ref().unwrap().text.len();

        // Push the unchanged part of the content
        new_content.push_str(&content[last_end..line_start]);

        // Replace the matched parts within the line
        let line = mat.lines.as_ref().unwrap().text.clone();
        let replaced_line = re
          .replace_all(&line, |caps: &regex::Captures| {
            let matched_text = caps.get(0).unwrap().as_str();
            match replace_text_state.kind {
              ReplaceTextKind::PreserveCase => {
                let first_char = matched_text.chars().next().unwrap_or_default();
                if matched_text.chars().all(char::is_uppercase) {
                  replace_text_state.text.to_uppercase()
                } else if first_char.is_uppercase() {
                  replace_text_state
                    .text
                    .chars()
                    .enumerate()
                    .map(|(i, rc)| if i == 0 { rc.to_uppercase().to_string() } else { rc.to_lowercase().to_string() })
                    .collect::<String>()
                } else {
                  replace_text_state.text.to_lowercase()
                }
              },
              ReplaceTextKind::Simple => replace_text_state.text.to_string(),
            }
          })
          .to_string();

        // Add the modified line to the new content
        new_content.push_str(&replaced_line);

        last_end = line_end;
      }

      // Add the remaining content after the last match
      new_content.push_str(&content[last_end..]);

      // Write the new content back to the file
      let mut file = fs::OpenOptions::new().write(true).truncate(true).open(file_path).expect("Unable to open file");
      file.write_all(new_content.as_bytes()).expect("Unable to write file");
    }

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
    log::info!("Processing replace in {:?}", project_root);
    let force_replace = self.force_replace.0;
    let replace_text_state = store.select(|state: &State| state.replace_text.clone()).await;
    let search_text_state = store.select(|state: &State| state.search_text.clone()).await;
    if force_replace {
      self.handle_confirm(store.clone()).await;
    } else if search_text_state.text.is_empty() {
      let search_text_action =
        AppAction::Tui(TuiAction::Notify(NotificationEnum::Error("Search text cannot be empty".to_string())));
      self.command_tx.send(search_text_action).unwrap();

      return;
    } else if replace_text_state.text.is_empty() {
      let confirm_dialog = Action::SetDialog {
        dialog: Some(Dialog::ConfirmReplace(ConfirmDialog {
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

      // Here we return early, the dialog will handle the user input and trigger actions
      return;
    } else if is_git_repo(project_root) {
      self.handle_confirm(store.clone()).await;
    } else {
      let confirm_dialog = Action::SetDialog {
        dialog: Some(Dialog::ConfirmGitDirectory(ConfirmDialog {
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

      // Here we return early, the dialog will handle the user input and trigger actions
      return;
    }
  }
}
