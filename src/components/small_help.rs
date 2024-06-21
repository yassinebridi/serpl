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
  redux::{action::Action, state::State, thunk::ThunkAction},
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
    let content = if state.active_tab == Tab::Search {
      "Search: <Enter> search, <Tab> switch replace, <Ctrl-n> toggle search mode"
    } else if state.active_tab == Tab::Replace {
      "Replace: <C-o> replace, <Tab> switch search list, <Ctrl-n> toggle search mode"
    } else if state.active_tab == Tab::SearchResult {
      "Search List: <Enter> open file, <Tab> switch search, <j> go next, <k> go previous, <g> go top, <G> go bottom, <d> delete file"
    } else if state.active_tab == Tab::Preview {
      "Preview: <Enter> go back to list, <Tab> switch search, <j> go next, <k> go previous, <g> go top, <G> go bottom, <d> delete line"
    } else {
      "<Ctrl-C> exit"
    };

    let small_help = SmallHelpWidget::new(content.to_string(), Color::Blue, Alignment::Left);
    f.render_widget(small_help, layout.status_left);
    Ok(())
  }
}
