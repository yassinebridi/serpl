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
  layout::{self, get_layout, get_notification_layout},
  redux::{action::Action, state::State, thunk::ThunkAction},
  ripgrep::RipgrepOutput,
  tabs::Tab,
  ui::notification_box::NotificationBox,
};
const NOTIFICATION_DURATION: u64 = 3;

#[derive(Default)]
pub struct Notifications {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  notifications: Vec<NotificationWithTimestamp>,
}

pub type NotificationWithTimestamp = (NotificationEnum, Instant);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum NotificationEnum {
  Info(String),
  Warning(String),
  Error(String),
}

impl Notifications {
  pub fn new() -> Self {
    Self::default()
  }

  fn app_tick(&mut self) -> Result<()> {
    let now = Instant::now();
    self.notifications.retain(|(_, timestamp)| timestamp.elapsed().as_secs() < NOTIFICATION_DURATION);
    Ok(())
  }

  fn render_tick(&mut self) -> Result<()> {
    Ok(())
  }
}

impl Component for Notifications {
  fn update(&mut self, action: AppAction) -> Result<Option<AppAction>> {
    match action {
      AppAction::Tui(TuiAction::Tick) => {
        self.app_tick()?;
      },
      AppAction::Tui(TuiAction::Render) => {
        self.render_tick()?;
      },
      AppAction::Tui(TuiAction::Notify(notification)) => {
        // FIFO push, don't exceed 4 notifications
        self.notifications.push((notification, Instant::now()));
        if self.notifications.len() > 4 {
          self.notifications.remove(0);
        }
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<()> {
    if !self.notifications.is_empty() {
      for (i, notification) in self.notifications.iter().enumerate() {
        let content = match &notification.0 {
          NotificationEnum::Info(s) => s,
          NotificationEnum::Warning(s) => s,
          NotificationEnum::Error(s) => s,
        };

        let notification_box = NotificationBox::new(notification, content);
        let rect = get_notification_layout(rect, content, i as u16);
        f.render_widget(notification_box, rect);
      }
    }
    Ok(())
  }
}
