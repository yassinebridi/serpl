use std::sync::Arc;

use async_trait::async_trait;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};
use tokio::sync::mpsc::UnboundedSender;

use super::process_search::ProcessSearchThunk;
use crate::{
  action::{AppAction, TuiAction},
  components::notifications::NotificationEnum,
  redux::{action::Action, state::State},
};

pub struct ToggleHiddenFilesThunk {
  command_tx: Arc<UnboundedSender<AppAction>>,
}

impl ToggleHiddenFilesThunk {
  pub fn new(command_tx: Arc<UnboundedSender<AppAction>>) -> Self {
    Self { command_tx }
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for ToggleHiddenFilesThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let include_hidden = !store.select(|state: &State| state.include_hidden).await;
    store.dispatch(Action::SetIncludeHidden { include_hidden }).await;

    let message =
      if include_hidden { "Hidden files included. Click 'Enter' to search" } else { "Hidden files excluded" };
    let _ = self.command_tx.send(AppAction::Tui(TuiAction::Notify(NotificationEnum::Info(message.to_string()))));

    ProcessSearchThunk::new().execute(store).await;
  }
}
