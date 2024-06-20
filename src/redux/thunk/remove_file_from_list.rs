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

pub struct RemoveFileFromListThunk {
  pub index: usize,
}

impl RemoveFileFromListThunk {
  pub fn new(index: usize) -> Self {
    Self { index }
  }
}

impl Default for RemoveFileFromListThunk {
  fn default() -> Self {
    Self::new(0)
  }
}

#[async_trait]
impl<Api> Thunk<State, Action, Api> for RemoveFileFromListThunk
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  async fn execute(&self, store: Arc<Api>) {
    // Get the current state
    let search_list = store.select(|state: &State| state.search_result.clone()).await;

    // Ensure the index is within bounds
    if self.index < search_list.list.len() {
      // Remove the file from the list in the state
      let mut updated_list = search_list.list.clone();
      updated_list.remove(self.index);

      // Update the state with the new list
      let updated_search_list = SearchListState { list: updated_list.clone(), ..search_list };
      store.dispatch(Action::SetSearchList { search_list: updated_search_list }).await;

      // Update the selected result to None or the next available item
      let new_selected_result = if updated_list.is_empty() {
        SearchResultState::default()
      } else {
        updated_list[self.index.min(updated_list.len() - 1)].clone()
      };
      store.dispatch(Action::SetSelectedResult { result: new_selected_result }).await;
    }
  }
}
