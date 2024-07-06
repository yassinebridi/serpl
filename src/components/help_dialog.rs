use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
  action::AppAction,
  config::{key_event_to_string, Config, KeyBindings},
  redux::{
    action::Action,
    state::{Dialog, State},
  },
  ui::help_display_dialog::{HelpDisplayDialogState, HelpDisplayDialogWidget, Tab},
};

#[derive(Default)]
pub struct HelpDialog {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  help_dialog_state: HelpDisplayDialogState,
  tabs: Vec<Tab>,
  active_tab: usize,
}

impl HelpDialog {
  pub fn new(config: Config) -> Self {
    let tabs =
      vec![Tab { title: "[1] Global".to_string(), content: Self::global_keybindings(&config.keybindings) }, Tab {
        title: "[2] Navigation".to_string(),
        content: Self::navigation_keybindings(),
      }];
    Self { config, tabs, active_tab: 0, ..Default::default() }
  }

  // fn global_keybindings(keybindings: &KeyBindings) -> String {
  //   let mut content = String::from("Global Keybindings:\n");
  //
  //   for (key_events, action) in keybindings.iter() {
  //     let key_str = key_events.iter().map(key_event_to_string).collect::<Vec<_>>().join(", ");
  //
  //     content.push_str(&format!("- {}: {:?}\n", key_str, action));
  //   }
  //
  //   content
  // }

  fn global_keybindings(keybindings: &KeyBindings) -> String {
    "- q: Quit\n- Ctrl-c: Quit\n- Ctrl-u: Help dialog\n- Cltr-o: Process Replace\n- Ctrl-n: Loop through search and replace modes\n- Enter: Select/Deselect file\n- d: delete file/delete line from the replace process".to_string()
  }

  fn navigation_keybindings() -> String {
    "- Tab: Loop through panes\n- j/UpArrow: Move up\n- k/DownArrow: Move down\n- h/g/LeftArrow: Move to Top\n- l/G/RightArrow: Move to Bottom\n".to_string()
  }
}

impl Component for HelpDialog {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if let Some(Dialog::HelpDialog(_)) = &state.dialog {
      match (key.code, key.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
          let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
          self.command_tx.as_ref().unwrap().send(hide_dialog)?;
          self.help_dialog_state.show = false;
          Ok(None)
        },
        (KeyCode::Left, KeyModifiers::NONE)
        | (KeyCode::Char('h'), KeyModifiers::NONE)
        | (KeyCode::Tab, KeyModifiers::SHIFT) => {
          self.active_tab = if self.active_tab == 0 { self.tabs.len() - 1 } else { self.active_tab - 1 };
          Ok(None)
        },
        (KeyCode::Right, KeyModifiers::NONE)
        | (KeyCode::Char('l'), KeyModifiers::NONE)
        | (KeyCode::Tab, KeyModifiers::NONE) => {
          self.active_tab = (self.active_tab + 1) % self.tabs.len();
          Ok(None)
        },
        (KeyCode::Char('1'), KeyModifiers::NONE) => {
          self.active_tab = 0;
          Ok(None)
        },
        (KeyCode::Char('2'), KeyModifiers::NONE) => {
          self.active_tab = 1;
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
    if let Some(Dialog::HelpDialog(_)) = &state.dialog {
      let dialog_widget = HelpDisplayDialogWidget::new(self.tabs.clone(), self.active_tab);
      f.render_stateful_widget(dialog_widget, rect, &mut self.help_dialog_state);
    }
    Ok(())
  }
}
