use std::{fs, io::Write, process::Command};

use regex::Regex;

use super::{action::Action, state::State};
use crate::{
  mode::Mode,
  redux::state::{
    Dialog, FocusedScreen, NotificationState, ReplaceTextState, SearchListState, SearchResultState, SearchTextKind,
    SearchTextState,
  },
  tabs::Tab,
};

pub fn reducer(state: State, action: Action) -> State {
  match action {
    Action::SetSearchList { search_list } => State { search_result: search_list, ..state },
    Action::SetSelectedResult { result } => State { selected_result: result, ..state },
    Action::SetSearchText { text } => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };
      if is_dialog_visible {
        return state;
      }
      let search_kind = &state.search_text.kind;
      State { search_text: SearchTextState { text, kind: search_kind.clone() }, ..state }
    },
    Action::SetReplaceText { text } => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };
      if is_dialog_visible {
        return state;
      }
      let replace_kind = &state.replace_text.kind;
      State { replace_text: ReplaceTextState { text, kind: replace_kind.clone() }, ..state }
    },
    Action::SetSearchTextKind { kind } => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };
      if is_dialog_visible {
        return state;
      }
      State { search_text: SearchTextState { kind, text: state.search_text.text.clone() }, ..state }
    },
    Action::SetReplaceTextKind { kind } => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };
      if is_dialog_visible {
        return state;
      }
      State { replace_text: ReplaceTextState { kind, text: state.replace_text.text.clone() }, ..state }
    },
    Action::SetActiveTab { tab } => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };

      if is_dialog_visible {
        return state;
      }
      State {
        active_tab: tab,
        focused_screen: match tab {
          Tab::Search => FocusedScreen::SearchInput,
          Tab::Replace => FocusedScreen::ReplaceInput,
          Tab::SearchResult => FocusedScreen::SearchResultList,
          Tab::Preview => FocusedScreen::Preview,
        },
        ..state
      }
    },
    Action::LoopOverTabs => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };

      if is_dialog_visible {
        return state;
      }
      State {
        active_tab: match state.active_tab {
          Tab::Search => Tab::Replace,
          Tab::Replace => Tab::SearchResult,
          Tab::SearchResult => Tab::Search,
          Tab::Preview => Tab::Preview,
        },
        focused_screen: match state.active_tab {
          Tab::Search => FocusedScreen::ReplaceInput,
          Tab::Replace => FocusedScreen::SearchResultList,
          Tab::SearchResult => FocusedScreen::SearchInput,
          Tab::Preview => FocusedScreen::Preview,
        },
        ..state
      }
    },
    Action::BackLoopOverTabs => {
      let is_dialog_visible = match &state.dialog {
        Some(dialog) => {
          match dialog {
            Dialog::ConfirmGitDirectory(dialog) => dialog.show,
            Dialog::ConfirmReplace(dialog) => dialog.show,
          }
        },
        None => false,
      };

      if is_dialog_visible {
        return state;
      }
      State {
        active_tab: match state.active_tab {
          Tab::Search => Tab::SearchResult,
          Tab::Replace => Tab::Search,
          Tab::SearchResult => Tab::Replace,
          Tab::Preview => Tab::Preview,
        },
        focused_screen: match state.active_tab {
          Tab::Search => FocusedScreen::SearchResultList,
          Tab::Replace => FocusedScreen::SearchInput,
          Tab::SearchResult => FocusedScreen::ReplaceInput,
          Tab::Preview => FocusedScreen::Preview,
        },
        ..state
      }
    },
    Action::ChangeMode { mode } => State { mode, ..state },
    Action::SetGlobalLoading { global_loading } => State { global_loading, ..state },
    Action::ResetState => State::new(state.project_root.clone()),
    Action::SetNotification { message, show, ttl, color } => {
      State { notification: NotificationState { message, show, ttl, color }, ..state }
    },
    Action::SetDialog { dialog } => {
      let temporary_dialog = dialog.clone();
      State {
        dialog,
        focused_screen: match temporary_dialog {
          Some(Dialog::ConfirmGitDirectory(_)) => FocusedScreen::ConfirmGitDirectoryDialog,
          Some(Dialog::ConfirmReplace(_)) => FocusedScreen::ConfirmReplaceDialog,
          _ => FocusedScreen::SearchInput,
        },
        ..state
      }
    },
  }
}
