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

impl Default for RemoveLineFromFileThunk {
  fn default() -> Self {
    Self::new(0, 0)
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for RemoveLineFromFileThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    // Get the current state
    let search_list = store.select(|state: &State| state.search_result.clone()).await;

    // Ensure the indices are within bounds
    if self.file_index < search_list.list.len() {
      let mut updated_list = search_list.list.clone();
      if self.line_index < updated_list[self.file_index].matches.len() {
        updated_list[self.file_index].matches.remove(self.line_index);

        // Update total_matches
        updated_list[self.file_index].total_matches -= 1;

        // Update the state with the new list
        let updated_search_list = SearchListState { list: updated_list.clone(), ..search_list };
        store.dispatch(Action::SetSearchList { search_list: updated_search_list }).await;

        // Update the selected result to None or the next available item
        let new_selected_result =
          if updated_list.is_empty() { SearchResultState::default() } else { updated_list[self.file_index].clone() };
        store.dispatch(Action::SetSelectedResult { result: new_selected_result }).await;
      }
    }
  }
}
