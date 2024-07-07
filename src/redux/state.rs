use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  time::{Duration, SystemTime},
};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::{mode::Mode, ripgrep::RipgrepLines, tabs::Tab};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct State {
  pub search_result: SearchListState,
  pub selected_result: SearchResultState,
  pub search_text: SearchTextState,
  pub replace_text: ReplaceTextState,
  pub active_tab: Tab,
  pub mode: Mode,
  pub global_loading: bool,
  pub notification: NotificationState,
  pub dialog: Option<Dialog>,
  pub project_root: PathBuf,
  pub focused_screen: FocusedScreen,
  pub previous_focused_screen: FocusedScreen,
  pub help_dialog_visible: bool,
  pub is_large_folder: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum FocusedScreen {
  #[default]
  SearchInput,
  ReplaceInput,
  SearchResultList,
  Preview,
  ConfirmGitDirectoryDialog,
  ConfirmReplaceDialog,
  HelpDialog,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SearchTextState {
  pub text: String,
  pub kind: SearchTextKind,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq, Copy)]
pub enum SearchTextKind {
  #[default]
  Simple,
  MatchCase,
  MatchWholeWord,
  MatchCaseWholeWord,
  Regex,
  AstGrep,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ReplaceTextState {
  pub text: String,
  pub kind: ReplaceTextKind,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq, Copy)]
pub enum ReplaceTextKind {
  #[default]
  Simple,
  PreserveCase,
  AstGrep,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Dialog {
  ConfirmGitDirectory(ConfirmDialogState),
  ConfirmReplace(ConfirmDialogState),
  HelpDialog(HelpDialogState),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HelpDialogState {
  pub show: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConfirmDialogState {
  pub message: String,
  pub on_confirm: Option<DialogAction>,
  pub on_cancel: Option<DialogAction>,
  pub confirm_label: String,
  pub cancel_label: String,
  pub show_cancel: bool,
  pub show: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DialogAction {
  ConfirmReplace,
  CancelReplace,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NotificationState {
  pub message: String,
  pub show: bool,
  pub ttl: u64,
  pub color: Color,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SearchListState {
  pub list: Vec<SearchResultState>,
  pub metadata: Metadata,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Metadata {
  pub elapsed_time: u64,
  pub matched_lines: usize,
  pub matches: usize,
  pub searches: usize,
  pub searches_with_match: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct SearchResultState {
  pub index: Option<usize>,
  pub path: String,
  pub matches: Vec<Match>,
  pub total_matches: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct Match {
  pub line_number: usize,
  pub lines: Option<RipgrepLines>,
  pub context_before: Vec<String>,
  pub context_after: Vec<String>,
  pub absolute_offset: usize,
  pub submatches: Vec<SubMatch>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct SubMatch {
  pub start: usize,
  pub end: usize,
}

impl State {
  pub fn new(project_root: PathBuf) -> Self {
    Self { project_root, is_large_folder: false, ..Default::default() }
  }
}
