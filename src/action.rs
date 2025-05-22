use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

use crate::{
  action,
  components::notifications::NotificationEnum,
  mode::Mode,
  redux::{
    action::Action,
    state::{Dialog, HelpDialogState},
    thunk::{self, ForceReplace, ThunkAction},
    ActionOrThunk,
  },
  tabs::Tab,
};

#[derive(Display, Clone, Debug, PartialEq)]
pub enum AppAction {
  Tui(TuiAction),
  Action(action::Action),
  Thunk(thunk::ThunkAction),
}

#[derive(Display, Debug, Clone, PartialEq)]
pub enum TuiAction {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  Refresh,
  Error(String),
  Help,

  Notify(NotificationEnum),
  Status(String),
  Reset,
}

impl<'de> Deserialize<'de> for AppAction {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
      type Value = AppAction;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string representation of Action")
      }

      fn visit_str<E>(self, value: &str) -> Result<AppAction, E>
      where
        E: de::Error,
      {
        match value {
          // -- custom actions
          // "InputMode" => Ok(TuiAction::ModeChange(Mode::Input)),
          // "NormalMode" => Ok(TuiAction::ModeChange(Mode::Normal)),
          // "Up" => Ok(TuiAction::Up),
          // "Down" => Ok(TuiAction::Down),
          // "Left" => Ok(TuiAction::Left),
          // "Right" => Ok(TuiAction::Right),
          // "Tab" => Ok(TuiAction::Tab),
          // "BackTab" => Ok(TuiAction::BackTab),

          // -- default actions
          "Tick" => Ok(AppAction::Tui(TuiAction::Tick)),
          "Render" => Ok(AppAction::Tui(TuiAction::Render)),
          "Suspend" => Ok(AppAction::Tui(TuiAction::Suspend)),
          "Resume" => Ok(AppAction::Tui(TuiAction::Resume)),
          "Quit" => Ok(AppAction::Tui(TuiAction::Quit)),
          "Refresh" => Ok(AppAction::Tui(TuiAction::Refresh)),
          "Help" => Ok(AppAction::Tui(TuiAction::Help)),
          data if data.starts_with("Error(") => {
            let error_msg = data.trim_start_matches("Error(").trim_end_matches(')');
            Ok(AppAction::Tui(TuiAction::Error(error_msg.to_string())))
          },
          data if data.starts_with("Resize(") => {
            let parts: Vec<&str> = data.trim_start_matches("Resize(").trim_end_matches(')').split(',').collect();
            if parts.len() == 2 {
              let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
              let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
              Ok(AppAction::Tui(TuiAction::Resize(width, height)))
            } else {
              Err(E::custom(format!("Invalid Resize format: {}", value)))
            }
          },
          // Redux actions
          "LoopOverTabs" => Ok(AppAction::Action(Action::LoopOverTabs)),
          "BackLoopOverTabs" => Ok(AppAction::Action(Action::BackLoopOverTabs)),
          "SearchTab" => Ok(AppAction::Action(Action::SetActiveTab { tab: Tab::Search })),
          "ReplaceTab" => Ok(AppAction::Action(Action::SetActiveTab { tab: Tab::Replace })),
          "SearchResultTab" => Ok(AppAction::Action(Action::SetActiveTab { tab: Tab::SearchResult })),
          "InputMode" => Ok(AppAction::Action(Action::ChangeMode { mode: Mode::Input })),
          "NormalMode" => Ok(AppAction::Action(Action::ChangeMode { mode: Mode::Normal })),
          "ShowHelp" => Ok(AppAction::Action(Action::SetDialog {
            dialog: Some(Dialog::HelpDialog(HelpDialogState { show: true })),
          })),
          // Redux Thunk Actions
          "ProcessReplace" => Ok(AppAction::Thunk(ThunkAction::ProcessReplace(ForceReplace(false)))),
          _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
        }
      }
    }

    deserializer.deserialize_str(ActionVisitor)
  }
}
