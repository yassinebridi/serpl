use std::{fs, sync::Arc};

use async_trait::async_trait;
use redux_rs::{middlewares::thunk::Thunk, StoreApi};
use regex::RegexBuilder;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::{AppAction, TuiAction},
  components::notifications::NotificationEnum,
  redux::{
    action::Action,
    state::{ReplaceTextKind, SearchTextKind, State},
    thunk::ThunkAction,
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
        let content = fs::read_to_string(file_path).expect("Unable to read file");
        let mut lines: Vec<String> = content.lines().map(String::from).collect();

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
          #[cfg(feature = "ast_grep")]
          SearchTextKind::AstGrep => panic!("AST-grep is not supported for line-by-line replacement"),
        };

        if let Some(line) = lines.get_mut(match_info.line_number - 1) {
          let replaced_line = re.replace_all(line, |caps: &regex::Captures| {
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
              #[cfg(feature = "ast_grep")]
              ReplaceTextKind::AstGrep => panic!("AST-grep is not supported for line-by-line replacement"),
            }
          });
          *line = replaced_line.into_owned();
        }

        let new_content = lines.join("\n");
        fs::write(file_path, new_content).expect("Unable to write file");
      }
    }
  }
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
