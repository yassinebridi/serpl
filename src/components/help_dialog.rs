use std::{
  collections::{HashMap, HashSet},
  process::Command,
  time::{Duration, Instant},
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    state::{Dialog, DialogAction, State},
    thunk::{ForceReplace, ThunkAction},
  },
  ripgrep::RipgrepOutput,
  tabs::Tab,
  ui::display_dialog::{DisplayDialogState, DisplayDialogWidget},
};

#[derive(Default)]
pub struct HelpDialog {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  help_dialog_state: DisplayDialogState,
}

impl HelpDialog {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for HelpDialog {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if let Some(Dialog::HelpDialog(dialog)) = &state.dialog {
      match (key.code, key.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Enter, KeyModifiers::NONE) => {
          let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
          self.command_tx.as_ref().unwrap().send(hide_dialog).unwrap();
          self.help_dialog_state.show = false;
          Ok(None)
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
    log::debug!("Drawing HelpDialog: {:?}", state.dialog);
    if let Some(Dialog::HelpDialog(dialog)) = &state.dialog {
      let keybindings = self.config.keybindings.clone();
       
      let help_text = r#"
        Global Keybindings:
        Keybindings:                                    | Navigation:
        - Ctrl + q: Quit                                | - Use arrow keys to navigate
        - Ctrl + f: Search                              | - Use tab to move to the next widget
        - Ctrl + r: Replace                             | - Use backtab to move to the previous widget
        - Ctrl + s: Save                                | - Use enter to select an item
        - Ctrl + n: Next tab                            | - Use esc to go back
        - Ctrl + p: Previous tab                        |
        - Ctrl + h: Show help dialog                    |
        - Ctrl + l: Loop over tabs                      |
        - Ctrl + b: Back loop over tabs                 |
        - Ctrl + g: Go to tab                           |
        - Ctrl + t: Toggle tab                          |
        - Ctrl + u: Scroll up                           |
      "#;

      let dialog_widget = DisplayDialogWidget::new("Help".to_string(), help_text.to_string());

      if dialog.show {
        f.render_stateful_widget(dialog_widget, rect, &mut self.help_dialog_state);
      }
    }
    Ok(())
  }
}
