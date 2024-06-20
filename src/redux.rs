pub mod action;
pub mod reducer;
pub mod state;
pub mod thunk;

#[derive(Debug)]
pub enum ActionOrThunk {
  Action(action::Action),
  Thunk(thunk::ThunkAction),
}

impl From<action::Action> for ActionOrThunk {
  fn from(action: action::Action) -> Self {
    Self::Action(action)
  }
}

impl From<thunk::ThunkAction> for ActionOrThunk {
  fn from(thunk: thunk::ThunkAction) -> Self {
    Self::Thunk(thunk)
  }
}
