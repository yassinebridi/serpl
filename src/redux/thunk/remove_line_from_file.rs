use std::sync::Arc;

use async_trait::async_trait;
use redux_rs::{
  middlewares::thunk::{self, Thunk},
  StoreApi,
};

use crate::redux::{
  action::Action,
  state::{SearchListState, SearchResultState, State},
};

pub struct RemoveLineFromFileThunk {
  pub file_index: usize,
  pub line_index: usize,
}

impl RemoveLineFromFileThunk {
  pub fn new(file_index: usize, line_index: usize) -> Self {
    Self { file_index, line_index }
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for RemoveLineFromFileThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    let mut search_list = store.select(|state: &State| state.search_result.clone()).await;

    if self.file_index < search_list.list.len() {
      let file_result = &mut search_list.list[self.file_index];
      if self.line_index < file_result.matches.len() {
        file_result.matches.remove(self.line_index);
        file_result.total_matches -= 1;

        if file_result.matches.is_empty() {
          search_list.list.remove(self.file_index);
        }

        store.dispatch(Action::SetSearchList { search_list: search_list.clone() }).await;

        if !search_list.list.is_empty() {
          let new_selected_index = self.file_index.min(search_list.list.len() - 1);
          let new_selected_result = search_list.list[new_selected_index].clone();
          store.dispatch(Action::SetSelectedResult { result: new_selected_result }).await;
        } else {
          store.dispatch(Action::SetSelectedResult { result: SearchResultState::default() }).await;
        }
      }
    }
  }
}
