use std::{fmt, string::ToString};

use ratatui::style::Color;
use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

use crate::{
  mode::Mode,
  redux::state::{Dialog, FocusedScreen, ReplaceTextKind, SearchListState, SearchResultState, SearchTextKind},
  tabs::Tab,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
  SetSearchList { search_list: SearchListState },
  SetSelectedResult { result: SearchResultState },
  SetSearchText { text: String },
  SetReplaceText { text: String },
  SetSearchTextKind { kind: SearchTextKind },
  SetReplaceTextKind { kind: ReplaceTextKind },
  SetActiveTab { tab: Tab },
  LoopOverTabs,
  BackLoopOverTabs,
  ChangeMode { mode: Mode },
  SetGlobalLoading { global_loading: bool },
  ResetState,
  SetNotification { message: String, show: bool, ttl: u64, color: Color },
  SetDialog { dialog: Option<Dialog> },
  SetFocusedScreen { screen: Option<FocusedScreen> },
  RemoveFileFromList { index: usize },
  RemoveLineFromFile { file_index: usize, line_index: usize },
  UpdateSearchResultFilter(String),
}
