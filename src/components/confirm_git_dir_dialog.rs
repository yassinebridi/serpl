use std::{
  collections::{HashMap, HashSet},
  process::Command,
  time::{Duration, Instant},
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::Position,
  prelude::*,
  widgets::{block::Title, *},
};
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{event, trace, Level};

use super::{Component, Frame};
use crate::{
  action::{AppAction, TuiAction},
  config::{Config, KeyBindings},
  redux::{
    action::Action,
    state::{ConfirmDialog, Dialog, DialogAction, State},
    thunk::{ForceReplace, ThunkAction},
  },
  ripgrep::RipgrepOutput,
  tabs::Tab,
  ui::confirm_dialog_widget::{ConfirmDialogAction, ConfirmDialogState, ConfirmDialogWidget},
};

#[derive(Default)]
pub struct ConfirmGitDirDialog {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  dialog_state: ConfirmDialogState,
}

impl ConfirmGitDirDialog {
  pub fn new() -> Self {
    Self::default()
  }

  fn handle_input(&self, action: DialogAction, state: &State) {
    match action {
      DialogAction::ConfirmReplace => {
        let process_replace_action = AppAction::Thunk(ThunkAction::ProcessReplace(ForceReplace(true)));
        self.command_tx.as_ref().unwrap().send(process_replace_action).unwrap();

        let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
        self.command_tx.as_ref().unwrap().send(hide_dialog).unwrap();
      },
      DialogAction::CancelReplace => {
        let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
        self.command_tx.as_ref().unwrap().send(hide_dialog).unwrap();
      },
    }
  }
}

impl Component for ConfirmGitDirDialog {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if let Some(Dialog::ConfirmGitDirectory(dialog)) = &state.dialog {
      match key.code {
        KeyCode::Tab
        | KeyCode::Down
        | KeyCode::Up
        | KeyCode::Right
        | KeyCode::Left
        | KeyCode::BackTab
        | KeyCode::Char('j')
        | KeyCode::Char('k')
        | KeyCode::Char('h')
        | KeyCode::Char('l') => {
          self.dialog_state.loop_selected_button();
          Ok(None)
        },
        KeyCode::Enter | KeyCode::Char('y') => {
          if let Some(action) = &dialog.on_confirm {
            match self.dialog_state.selected_button {
              ConfirmDialogAction::Confirm => {
                self.handle_input(action.clone(), state);
              },
              ConfirmDialogAction::Cancel => {
                let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
                self.command_tx.as_ref().unwrap().send(hide_dialog).unwrap();
              },
            }
            Ok(None)
          } else {
            Ok(None)
          }
        },
        KeyCode::Esc | KeyCode::Char('n') => {
          if let Some(action) = &dialog.on_cancel {
            self.handle_input(action.clone(), state);
            Ok(None)
          } else {
            Ok(None)
          }
        },
        _ => Ok(None),
      }
    } else {
      Ok(None)
    }
  }

  fn update(&mut self, action: AppAction) -> Result<Option<AppAction>> {
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<()> {
    if let Some(Dialog::ConfirmGitDirectory(dialog)) = &state.dialog {
      let dialog_widget = ConfirmDialogWidget::new(
        "Confirm Dialog".to_string(),
        dialog.message.clone(),
        dialog.confirm_label.clone(),
        dialog.cancel_label.clone(),
        dialog.show_cancel,
      );

      if dialog.show {
        f.render_stateful_widget(dialog_widget, rect, &mut self.dialog_state);
      }
    }
    Ok(())
  }
}
