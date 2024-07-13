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
use tokio::{sync::mpsc::UnboundedSender, time::Instant};
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
    state::{FocusedScreen, ReplaceTextKind, SearchResultState, SearchTextKind, State},
    thunk::ThunkAction,
  },
  ripgrep::RipgrepOutput,
  tabs::Tab,
};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct Search {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  input: Input,
  debounce_timer: Option<tokio::task::JoinHandle<()>>,
}

impl Search {
  pub fn new() -> Self {
    Self::default()
  }

  fn set_selected_result(&mut self, state: &State) {
    let first_result = match state.search_result.list.first() {
      Some(result) => result.clone(),
      None => SearchResultState::default(),
    };
    let selected_result = AppAction::Action(Action::SetSelectedResult { result: first_result.clone() });
    self.command_tx.as_ref().unwrap().send(selected_result).unwrap();
  }

  fn handle_input(&mut self, key: KeyEvent, state: &State) {
    let query = self.input.value();

    if let Some(timer) = self.debounce_timer.take() {
      timer.abort();
    }

    let tx = self.command_tx.clone().unwrap();
    let search_text_action = AppAction::Action(Action::SetSearchText { text: query.to_string() });
    let process_search_thunk = AppAction::Thunk(ThunkAction::ProcessSearch);

    if state.is_large_folder && key.code != KeyCode::Enter {
      tx.send(search_text_action).unwrap();
    } else if !state.is_large_folder || key.code == KeyCode::Enter {
      self.debounce_timer = Some(tokio::spawn(async move {
        tokio::time::sleep(DEBOUNCE_DURATION).await;
        tx.send(search_text_action).unwrap();
        tx.send(process_search_thunk).unwrap();
      }));
    }
  }

  fn change_kind(&mut self, search_text_kind: SearchTextKind, state: &State) {
    let search_text_action = AppAction::Action(Action::SetSearchTextKind { kind: search_text_kind });
    self.command_tx.as_ref().unwrap().send(search_text_action).unwrap();

    #[cfg(feature = "ast_grep")]
    if search_text_kind == SearchTextKind::AstGrep {
      let replace_text_action = AppAction::Action(Action::SetReplaceTextKind { kind: ReplaceTextKind::AstGrep });
      self.command_tx.as_ref().unwrap().send(replace_text_action).unwrap();
    }

    let process_search_thunk = AppAction::Thunk(ThunkAction::ProcessSearch);
    self.command_tx.as_ref().unwrap().send(process_search_thunk).unwrap();
    self.set_selected_result(state);
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
        (KeyCode::Tab, _) | (KeyCode::BackTab, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => Ok(None),
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
          #[cfg(feature = "ast_grep")]
          let search_text_kind = match state.search_text.kind {
            SearchTextKind::Simple => SearchTextKind::MatchCase,
            SearchTextKind::MatchCase => SearchTextKind::MatchWholeWord,
            SearchTextKind::MatchWholeWord => SearchTextKind::MatchCaseWholeWord,
            SearchTextKind::MatchCaseWholeWord => SearchTextKind::Regex,
            SearchTextKind::Regex => SearchTextKind::AstGrep,
            SearchTextKind::AstGrep => SearchTextKind::Simple,
          };
          #[cfg(not(feature = "ast_grep"))]
          let search_text_kind = match state.search_text.kind {
            SearchTextKind::Simple => SearchTextKind::MatchCase,
            SearchTextKind::MatchCase => SearchTextKind::MatchWholeWord,
            SearchTextKind::MatchWholeWord => SearchTextKind::MatchCaseWholeWord,
            SearchTextKind::MatchCaseWholeWord => SearchTextKind::Regex,
            SearchTextKind::Regex => SearchTextKind::Simple,
          };
          self.change_kind(search_text_kind, state);
          Ok(None)
        },
        (KeyCode::Enter, _) => {
          self.handle_input(key, state);
          Ok(None)
        },
        (KeyCode::Char(_c), _) => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          let key_bindings = self.config.keybindings.clone();
          let quit_keys = find_keys_for_value(&key_bindings.0, AppAction::Tui(TuiAction::Quit));
          if !is_quit_key(&quit_keys, &key) {
            self.handle_input(key, state);
          }
          Ok(None)
        },
        (KeyCode::Backspace | KeyCode::Delete, _) => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          let key_bindings = self.config.keybindings.clone();
          let quit_keys = find_keys_for_value(&key_bindings.0, AppAction::Tui(TuiAction::Quit));
          if !is_quit_key(&quit_keys, &key) {
            self.handle_input(key, state);
          }
          Ok(None)
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
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
      #[cfg(feature = "ast_grep")]
      SearchTextKind::AstGrep => "[AST Grep]",
    };

    let block = Block::bordered()
      .border_type(BorderType::Rounded)
      .title(Title::from("Search").alignment(Alignment::Left))
      .title(Title::from(search_kind).alignment(Alignment::Right));

    let block = if state.focused_screen == FocusedScreen::SearchInput {
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

    if state.focused_screen == FocusedScreen::SearchInput {
      f.set_cursor(
        layout.search_input.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
        layout.search_input.y + 1,
      );
    };

    f.render_widget(search_widget, layout.search_input);
    Ok(())
  }
}

fn find_keys_for_value(
  key_bindings: &HashMap<Vec<KeyEvent>, AppAction>,
  quit: AppAction,
) -> Option<Vec<Vec<KeyEvent>>> {
  let mut quit_keys = Vec::new();
  for (key, value) in key_bindings.iter() {
    if value == &quit {
      quit_keys.push(key.clone());
    }
  }
  if quit_keys.is_empty() {
    None
  } else {
    Some(quit_keys)
  }
}

fn is_quit_key(quit_keys: &Option<Vec<Vec<KeyEvent>>>, key: &KeyEvent) -> bool {
  if let Some(quit_keys) = quit_keys {
    for keys in quit_keys {
      if keys.contains(key) {
        return true;
      }
    }
  }
  false
}
