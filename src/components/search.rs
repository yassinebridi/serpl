use std::{
  collections::{HashMap, HashSet},
  process::Command,
  time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
  layout::Position,
  prelude::*,
  widgets::{block::Title, *},
};
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
    state::{FocusedScreen, SearchResultState, SearchTextKind, State},
    thunk::ThunkAction,
  },
  ripgrep::RipgrepOutput,
  tabs::Tab,
  utils::is_git_repo,
};

#[derive(Default)]
pub struct Search {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  input: Input,
}

impl Search {
  pub fn new() -> Self {
    Self::default()
  }

  fn handle_input(&mut self, key: KeyEvent) {
    let query = self.input.value();
    let search_text_action = AppAction::Action(Action::SetSearchText { text: query.to_string() });
    let process_search_thunk = AppAction::Thunk(ThunkAction::ProcessSearch);
    self.command_tx.as_ref().unwrap().send(search_text_action).unwrap();
    self.command_tx.as_ref().unwrap().send(process_search_thunk).unwrap();
  }

  fn change_kind(&mut self, search_text_kind: SearchTextKind) {
    let search_text_action = AppAction::Action(Action::SetSearchTextKind { kind: search_text_kind });
    self.command_tx.as_ref().unwrap().send(search_text_action).unwrap();
    let process_search_thunk = AppAction::Thunk(ThunkAction::ProcessSearch);
    self.command_tx.as_ref().unwrap().send(process_search_thunk).unwrap();
  }
}

impl Component for Search {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if state.focused_screen == FocusedScreen::SearchInput {
      match (key.code, key.modifiers) {
        (KeyCode::Tab, _) | (KeyCode::BackTab, _) => Ok(None),
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
          let search_text_kind = match state.search_text.kind {
            SearchTextKind::Simple => SearchTextKind::MatchCase,
            SearchTextKind::MatchCase => SearchTextKind::MatchWholeWord,
            SearchTextKind::MatchWholeWord => SearchTextKind::MatchCaseWholeWord,
            SearchTextKind::MatchCaseWholeWord => SearchTextKind::Regex,
            SearchTextKind::Regex => SearchTextKind::Simple,
          };
          self.change_kind(search_text_kind);
          Ok(None)
        },
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
          let search_text_kind = match state.search_text.kind {
            SearchTextKind::Simple => SearchTextKind::Regex,
            SearchTextKind::MatchCase => SearchTextKind::Simple,
            SearchTextKind::MatchWholeWord => SearchTextKind::MatchCase,
            SearchTextKind::MatchCaseWholeWord => SearchTextKind::MatchWholeWord,
            SearchTextKind::Regex => SearchTextKind::MatchCaseWholeWord,
          };
          self.change_kind(search_text_kind);
          Ok(None)
        },
        (KeyCode::Enter, _) => {
          self.handle_input(key);
          Ok(None)
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          let is_git_folder = is_git_repo(state.project_root.clone());
          if is_git_folder {
            self.handle_input(key);
          }
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
    let search_kind = match state.search_text.kind {
      SearchTextKind::Simple => "[Simple]",
      SearchTextKind::MatchCase => "[Match Case]",
      SearchTextKind::MatchWholeWord => "[Match Whole Word]",
      SearchTextKind::Regex => "[Regex]",
      SearchTextKind::MatchCaseWholeWord => "[Match Case Whole Word]",
    };

    let block = Block::bordered()
      .border_type(BorderType::Rounded)
      .title(Title::from("Search").alignment(Alignment::Left))
      .title(Title::from(search_kind).alignment(Alignment::Right));

    let block = if state.active_tab == Tab::Search {
      block.border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
      block
    };

    let width = layout.search_input.width.max(3) - 3;
    let scroll = self.input.visual_scroll(width as usize);

    let search_widget = Paragraph::new(self.input.value())
      .style(Style::default().fg(Color::White))
      .scroll((0, scroll as u16))
      .block(block);

    if state.active_tab == Tab::Search {
      f.set_cursor(
        layout.search_input.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
        layout.search_input.y + 1,
      );
    };

    f.render_widget(search_widget, layout.search_input);
    Ok(())
  }
}
