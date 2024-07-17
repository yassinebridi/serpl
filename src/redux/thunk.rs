use std::sync::Arc;

use redux_rs::{middlewares::thunk::Thunk, StoreApi};
use tokio::sync::mpsc::UnboundedSender;

use super::{action::Action, state::State};
use crate::action::{AppAction, TuiAction};

pub mod process_line_replace;
pub mod process_replace;
pub mod process_search;
pub mod process_single_file_replace;
pub mod remove_file_from_list;
pub mod remove_line_from_file;

#[derive(Debug, Clone, PartialEq)]
pub enum ThunkAction {
  ProcessSearch,
  ProcessReplace(ForceReplace),
  RemoveFileFromList(usize),
  RemoveLineFromFile(usize, usize),
  ProcessSingleFileReplace(usize),
  ProcessLineReplace(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceReplace(pub bool);

pub fn thunk_impl<Api>(
  action: ThunkAction,
  command_tx: Arc<UnboundedSender<AppAction>>,
) -> Box<dyn Thunk<State, Action, Api> + Send + Sync>
where
  Api: StoreApi<State, Action> + Send + Sync + 'static,
{
  match action {
    ThunkAction::ProcessSearch => Box::new(process_search::ProcessSearchThunk::new()),
    ThunkAction::ProcessReplace(force_replace) => {
      Box::new(process_replace::ProcessReplaceThunk::new(command_tx, force_replace))
    },
    ThunkAction::ProcessSingleFileReplace(index) => {
      Box::new(process_single_file_replace::ProcessSingleFileReplaceThunk::new(command_tx, index))
    },
    ThunkAction::ProcessLineReplace(file_index, line_index) => {
      Box::new(process_line_replace::ProcessLineReplaceThunk::new(command_tx, file_index, line_index))
    },
    ThunkAction::RemoveFileFromList(index) => Box::new(remove_file_from_list::RemoveFileFromListThunk::new(index)),
    ThunkAction::RemoveLineFromFile(file_index, line_index) => {
      Box::new(remove_line_from_file::RemoveLineFromFileThunk::new(file_index, line_index))
    },
  }
}
