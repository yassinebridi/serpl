use std::{fs, io::Write, process::Command};

use regex::Regex;

use super::{action::Action, state::State};
use crate::{
  mode::Mode,
  redux::{
    state::{
      Dialog, FocusedScreen, NotificationState, ReplaceTextState, SearchListState, SearchResultState, SearchTextKind,
      SearchTextState,
    },
    utils::selection_after_removals,
  },
  tabs::Tab,
};

pub fn reducer(state: State, action: Action) -> State {
  match action {
    Action::SetSearchList { search_list } => State { search_result: search_list, ..state },
    Action::SetSelectedResult { result } => State { selected_result: result, ..state },
    Action::SetSearchText { text } => {
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      let search_kind = &state.search_text.kind;
      State { search_text: SearchTextState { text, kind: *search_kind }, ..state }
    },
    Action::SetReplaceText { text } => {
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      let replace_kind = &state.replace_text.kind;
      State { replace_text: ReplaceTextState { text, kind: *replace_kind }, ..state }
    },
    Action::SetSearchTextKind { kind } => {
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      State { search_text: SearchTextState { kind, text: state.search_text.text.clone() }, ..state }
    },
    Action::SetReplaceTextKind { kind } => {
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      State { replace_text: ReplaceTextState { kind, text: state.replace_text.text.clone() }, ..state }
    },
    Action::SetActiveTab { tab } => {
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      State {
        active_tab: tab,
        previous_focused_screen: state.focused_screen,
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
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      State {
        previous_focused_screen: state.focused_screen,
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
      let is_dialog_visible = check_dialog_visible(&state);
      if is_dialog_visible {
        return state;
      }
      State {
        previous_focused_screen: state.focused_screen,
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
    Action::ResetState => State::new(state.project_root.clone()).with_include_hidden(state.include_hidden),
    Action::SetIncludeHidden { include_hidden } => State { include_hidden, ..state },
    Action::SetNotification { message, show, ttl, color } => {
      State { notification: NotificationState { message, show, ttl, color }, ..state }
    },
    Action::SetDialog { dialog } => {
      let temporary_dialog = dialog.clone();
      State {
        dialog,
        previous_focused_screen: state.focused_screen.clone(),
        focused_screen: match temporary_dialog {
          Some(Dialog::ConfirmGitDirectory(_)) => FocusedScreen::ConfirmGitDirectoryDialog,
          Some(Dialog::ConfirmReplace(_)) => FocusedScreen::ConfirmReplaceDialog,
          Some(Dialog::HelpDialog(_)) => FocusedScreen::HelpDialog,
          _ => state.focused_screen,
        },
        ..state
      }
    },
    Action::SetFocusedScreen { screen } => {
      State {
        previous_focused_screen: state.focused_screen,
        focused_screen: screen.unwrap_or(FocusedScreen::SearchInput),
        ..state
      }
    },
    Action::RemoveFileFromList { index } => {
      let mut new_search_result = state.search_result.clone();

      if index >= new_search_result.list.len() {
        return state;
      }

      let new_previewed_index =
        selection_after_removals(state.selected_result.index, &[index], new_search_result.list.len());
      new_search_result.list.remove(index);

      for (position, result) in new_search_result.list.iter_mut().enumerate() {
        result.index = Some(position);
      }

      let new_selected_result = match new_previewed_index {
        Some(position) => new_search_result.list[position].clone(),
        None => SearchResultState::default(),
      };

      State { search_result: new_search_result, selected_result: new_selected_result, ..state }
    },

    Action::RemoveLineFromFile { file_index, line_index } => {
      let mut new_search_result = state.search_result.clone();
      if file_index < new_search_result.list.len() {
        let file_result = &mut new_search_result.list[file_index];
        if line_index < file_result.matches.len() {
          let deleted_match = file_result.matches.remove(line_index);
          file_result.total_matches -= deleted_match.submatches.len();

          if file_result.matches.is_empty() {
            new_search_result.list.remove(file_index);
          }

          let new_selected_result = if !new_search_result.list.is_empty() {
            let new_selected_index = file_index.min(new_search_result.list.len() - 1);
            new_search_result.list[new_selected_index].clone()
          } else {
            SearchResultState::default()
          };

          State { search_result: new_search_result, selected_result: new_selected_result, ..state }
        } else {
          state
        }
      } else {
        state
      }
    },
    Action::ToggleInputMode => state,
  }
}

fn check_dialog_visible(state: &State) -> bool {
  match &state.dialog {
    Some(dialog) => {
      match dialog {
        Dialog::ConfirmGitDirectory(dialog) => dialog.show,
        Dialog::ConfirmReplace(dialog) => dialog.show,
        Dialog::HelpDialog(dialog) => dialog.show,
      }
    },
    None => false,
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  #[test]
  fn hidden_files_are_off_by_default() {
    assert!(!State::new(PathBuf::from(".")).include_hidden);
  }

  #[test]
  fn set_include_hidden_updates_state() {
    let state = reducer(State::default(), Action::SetIncludeHidden { include_hidden: true });
    assert!(state.include_hidden);
    let state = reducer(state, Action::SetIncludeHidden { include_hidden: false });
    assert!(!state.include_hidden);
  }

  #[test]
  fn reset_state_keeps_include_hidden() {
    let state = State::new(PathBuf::from("/x")).with_include_hidden(true);
    let state = reducer(state, Action::ResetState);
    assert!(state.include_hidden);
    assert_eq!(state.project_root, PathBuf::from("/x"));
  }

  #[test]
  fn manual_search_required_for_hidden_or_large_folder() {
    let mut state = State::default();
    assert!(!state.requires_manual_search());
    state.include_hidden = true;
    assert!(state.requires_manual_search());
    state.include_hidden = false;
    state.is_large_folder = true;
    assert!(state.requires_manual_search());
  }

  fn state_with_files(count: usize, previewed: usize) -> State {
    let mut state = State::default();
    state.search_result.list = (0..count)
      .map(|i| SearchResultState { index: Some(i), path: format!("f{i}.txt"), ..Default::default() })
      .collect();
    state.selected_result = state.search_result.list[previewed].clone();
    state
  }

  #[test]
  fn removing_a_file_above_the_previewed_one_keeps_the_preview() {
    let state = reducer(state_with_files(4, 2), Action::RemoveFileFromList { index: 0 });
    assert_eq!(state.selected_result.path, "f2.txt");
    assert_eq!(state.selected_result.index, Some(1));
    assert_eq!(state.search_result.list.iter().map(|r| r.index).collect::<Vec<_>>(), vec![Some(0), Some(1), Some(2)]);
  }

  #[test]
  fn removing_a_file_below_the_previewed_one_keeps_the_preview() {
    let state = reducer(state_with_files(4, 1), Action::RemoveFileFromList { index: 3 });
    assert_eq!(state.selected_result.path, "f1.txt");
    assert_eq!(state.selected_result.index, Some(1));
  }

  #[test]
  fn removing_the_previewed_file_falls_to_the_next_one() {
    let state = reducer(state_with_files(4, 2), Action::RemoveFileFromList { index: 2 });
    assert_eq!(state.selected_result.path, "f3.txt");
    assert_eq!(state.selected_result.index, Some(2));
  }

  #[test]
  fn removing_an_out_of_bound_index_is_a_no_op() {
    let before = state_with_files(2, 1);
    let state = reducer(before.clone(), Action::RemoveFileFromList { index: 2 });
    assert_eq!(state.search_result, before.search_result);
    assert_eq!(state.selected_result, before.selected_result);
  }
}
