use std::{
  collections::{HashMap, HashSet},
  process::Command,
  time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{event, trace, Level};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{
  action::{AppAction, TuiAction},
  components::notifications::NotificationEnum,
  config::{Config, KeyBindings},
  layout::get_layout,
  redux::{
    action::Action,
    state::{FocusedScreen, State},
    thunk::ThunkAction,
  },
  ripgrep::RipgrepOutput,
  tabs::Tab,
  ui::small_help_widget::SmallHelpWidget,
};

#[derive(Default)]
pub struct SmallHelp {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  input: Input,
}

impl SmallHelp {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for SmallHelp {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
    let layout = get_layout(area);
    let content = match state.focused_screen {
      FocusedScreen::SearchInput => "Help: <Ctrl-b> | Search: <Enter> | Switch to Replace: <Tab> | Toggle search mode: <Ctrl-n>",
      FocusedScreen::ReplaceInput => "Help: <Ctrl-b> | Replace: <C-o> | Switch to Search List: <Tab> | Toggle replace mode: <Ctrl-n>",
      FocusedScreen::SearchResultList => "Help: <Ctrl-b> | Open File: <Enter> | Switch to Search: <Tab> | Next: <j> | Previous: <k> | Top: <g> | Bottom: <G> | Delete file: <d>",
      FocusedScreen::Preview => "Help: <Ctrl-b> | Back to list: <Enter> | Switch to Search: <Tab> | Next: <j> | Previous: <k> | Top: <g> | Bottom: <G> | Delete line: <d>",
      FocusedScreen::ConfirmReplaceDialog => "Confirm Replace: <Enter> | Cancel Replace: <Esc>, Left: <h>, Right: <l>, Loop: <Tab>",
      FocusedScreen::ConfirmGitDirectoryDialog => "Confirm Replace: <Enter> | Cancel Replace: <Esc>, Left: <h>, Right: <l>, Loop: <Tab>",
      FocusedScreen::HelpDialog => "Close Help: <Esc> | Next Tab: <Right> | Previous Tab: <Left>",
    };

    let small_help = SmallHelpWidget::new(content.to_string(), Color::Blue, Alignment::Left);
    f.render_widget(small_help, layout.status_left);
    Ok(())
  }
}
