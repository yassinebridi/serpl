use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
  prelude::*,
  widgets::{block::Title, *},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{
  action::{AppAction, TuiAction},
  config::{Config, KeyBindings},
  layout::get_layout,
  redux::{
    action::Action,
    state::{ReplaceTextKind, State},
    thunk::ThunkAction,
  },
  tabs::Tab,
  utils::is_git_repo,
};

#[derive(Default)]
pub struct Replace {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  input: Input,
}

impl Replace {
  pub fn new() -> Self {
    Self::default()
  }

  fn handle_input(&mut self, key: KeyEvent) {
    self.input.handle_event(&crossterm::event::Event::Key(key));
    let query = self.input.value();
    let replace_text_action = AppAction::Action(Action::SetReplaceText { text: query.to_string() });
    self.command_tx.as_ref().unwrap().send(replace_text_action).unwrap();
  }

  fn change_kind(&mut self, replace_text_kind: ReplaceTextKind) {
    let replace_text_action = AppAction::Action(Action::SetReplaceTextKind { kind: replace_text_kind });
    self.command_tx.as_ref().unwrap().send(replace_text_action).unwrap();
    let process_search_thunk = AppAction::Thunk(ThunkAction::ProcessSearch);
    self.command_tx.as_ref().unwrap().send(process_search_thunk).unwrap();
  }
}

impl Component for Replace {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if state.active_tab == Tab::Replace {
      match (key.code, key.modifiers) {
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
          let replace_text_kind = match state.replace_text.kind {
            ReplaceTextKind::Simple => ReplaceTextKind::PreserveCase,
            ReplaceTextKind::PreserveCase => ReplaceTextKind::Simple,
          };
          Ok(None)
        },
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
          let replace_text_kind = match state.replace_text.kind {
            ReplaceTextKind::Simple => ReplaceTextKind::PreserveCase,
            ReplaceTextKind::PreserveCase => ReplaceTextKind::Simple,
          };
          self.change_kind(replace_text_kind);
          Ok(None)
        },
        _ => {
          self.handle_input(key);
          Ok(None)
        },
      }
    } else {
      Ok(None)
    }
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: AppAction) -> Result<Option<AppAction>> {
    if let AppAction::Tui(TuiAction::Reset) = action {
      self.input.reset()
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
    let layout = get_layout(area);

    let replace_kind = match state.replace_text.kind {
      ReplaceTextKind::Simple => "[Simple]",
      ReplaceTextKind::PreserveCase => "[Preserve Case]",
    };

    let block = Block::bordered()
      .border_type(BorderType::Rounded)
      .title(Title::from("Replace").alignment(Alignment::Left))
      .title(Title::from(replace_kind).alignment(Alignment::Right));

    let block = if state.active_tab == Tab::Replace {
      block.border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
      block
    };

    let width = layout.replace_input.width.max(3) - 3;
    let scroll = self.input.visual_scroll(width as usize);

    let replace_widget = Paragraph::new(self.input.value())
      .style(Style::default().fg(Color::White))
      .scroll((0, scroll as u16))
      .block(block);

    if state.active_tab == Tab::Replace {
      f.set_cursor(
        layout.replace_input.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
        layout.replace_input.y + 1,
      );
    };

    f.render_widget(replace_widget, layout.replace_input);
    Ok(())
  }
}
