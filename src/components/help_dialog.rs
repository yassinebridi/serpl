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
    state::{Dialog, FocusedScreen, HelpDialogState, State},
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
  pub fn new() -> Self {
    let tabs = vec![Tab { title: "[1] Global".to_string(), content: Self::global_keybindings() }, Tab {
      title: "[2] Navigation".to_string(),
      content: Self::navigation_keybindings(),
    }];
    Self { tabs, active_tab: 0, ..Default::default() }
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

  fn global_keybindings() -> String {
    "- q: Quit\n- Ctrl-c: Quit\n- Ctrl-b: Help dialog\n- Cltr-o: Process Replace For All Files\n- Ctrl-n: Loop through search and replace modes\n- Enter: Select/Deselect file\n- d: delete file/delete line from the replace process\n- r: Replace Selected File Or Line".to_string()
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
    if state.focused_screen == FocusedScreen::HelpDialog {
      match (key.code, key.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
          log::info!("Drawing HelpDialog 11{:?}", state.focused_screen);
          self.help_dialog_state.show = false;
          let previous_focused_screen = state.previous_focused_screen.clone();
          let hide_dialog = AppAction::Action(Action::SetDialog { dialog: None });
          self.command_tx.as_ref().unwrap().send(hide_dialog).unwrap();
          let focus_screen = AppAction::Action(Action::SetFocusedScreen { screen: Some(previous_focused_screen) });
          self.command_tx.as_ref().unwrap().send(focus_screen).unwrap();
          log::info!("Drawing HelpDialog 22{:?}", state.focused_screen);
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
    if let Some(Dialog::HelpDialog(HelpDialogState { show: true })) = &state.dialog {
      let dialog_widget = HelpDisplayDialogWidget::new(self.tabs.clone(), self.active_tab);
      f.render_stateful_widget(dialog_widget, rect, &mut self.help_dialog_state);
    }
    Ok(())
  }
}
