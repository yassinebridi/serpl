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
pub struct Status {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  input: Input,
  content: String,
}

impl Status {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for Status {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: AppAction) -> Result<Option<AppAction>> {
    if let AppAction::Tui(TuiAction::Status(content)) = action {
      self.content = content;
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
    let layout = get_layout(area);

    let small_help = SmallHelpWidget::new(self.content.clone(), Color::Yellow, Alignment::Right);
    f.render_widget(small_help, layout.status_right);
    Ok(())
  }
}
