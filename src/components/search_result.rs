use std::{collections::HashMap, default, time::Duration};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  prelude::*,
  style::Stylize,
  widgets::{block::Title, *},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::Input;

use super::{Component, Frame};
use crate::{
  action::AppAction,
  components::search_result,
  config::{Config, KeyBindings},
  layout::get_layout,
  redux::{
    action::Action,
    state::{FocusedScreen, SearchResultState, State},
    thunk::ThunkAction,
  },
  tabs::Tab,
};

#[derive(Default)]
pub struct SearchResult {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  state: ListState,
  match_counts: Vec<String>,
}

impl SearchResult {
  pub fn new() -> Self {
    Self::default()
  }

  fn delete_file(&mut self, state: &State) {
    if let Some(selected_index) = self.state.selected() {
      if selected_index < state.search_result.list.len() {
        let remove_file_from_list_thunk = AppAction::Thunk(ThunkAction::RemoveFileFromList(selected_index));
        self.command_tx.as_ref().unwrap().send(remove_file_from_list_thunk).unwrap();

        if state.search_result.list.len() > 1 {
          if selected_index >= state.search_result.list.len() - 1 {
            self.state.select(Some(state.search_result.list.len() - 2));
          } else {
            self.state.select(Some(selected_index));
          }
        } else {
          self.state.select(None);
        }

        self.update_selected_result(state);
      }
    }
  }

  fn next(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      return;
    }

    let new_index = match self.state.selected() {
      Some(i) => {
        if i >= state.search_result.list.len() - 1 {
          0
        } else {
          i + 1
        }
      },
      None => 0,
    };
    self.state.select(Some(new_index));
    self.update_selected_result(state);
  }

  fn previous(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      return;
    }

    let new_index = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          state.search_result.list.len() - 1
        } else {
          i - 1
        }
      },
      None => state.search_result.list.len() - 1,
    };
    self.state.select(Some(new_index));
    self.update_selected_result(state);
  }

  fn update_selected_result(&mut self, state: &State) {
    if let Some(selected_index) = self.state.selected() {
      if let Some(selected_result) = state.search_result.list.get(selected_index) {
        let action = AppAction::Action(Action::SetSelectedResult {
          result: SearchResultState {
            index: Some(selected_index),
            path: selected_result.path.clone(),
            matches: selected_result.matches.clone(),
            total_matches: selected_result.total_matches,
          },
        });
        self.command_tx.as_ref().unwrap().send(action).unwrap();
      } else {
        let action = AppAction::Action(Action::SetSelectedResult { result: SearchResultState::default() });
        self.command_tx.as_ref().unwrap().send(action).unwrap();
        self.state.select(None);
      }
    } else {
      let action = AppAction::Action(Action::SetSelectedResult { result: SearchResultState::default() });
      self.command_tx.as_ref().unwrap().send(action).unwrap();
    }
  }

  fn set_selected_result(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      self.state.select(None);
      return;
    }

    if let Some(selected_index) = self.state.selected() {
      if selected_index >= state.search_result.list.len() {
        self.state.select(Some(state.search_result.list.len() - 1));
      }
    } else {
      self.state.select(Some(0));
    }

    self.update_selected_result(state);
  }

  fn top(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      return;
    }

    self.state.select(Some(0));
    let selected_result = state.search_result.list.first().unwrap();
    let action = AppAction::Action(Action::SetSelectedResult {
      result: SearchResultState {
        index: selected_result.index,
        path: selected_result.path.clone(),
        matches: selected_result.matches.clone(),
        total_matches: selected_result.total_matches,
      },
    });
    self.command_tx.as_ref().unwrap().send(action).unwrap();
  }

  fn bottom(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      return;
    }

    let i = state.search_result.list.len() - 1;
    self.state.select(Some(i));
    let selected_result = state.search_result.list.get(i).unwrap();
    let action = AppAction::Action(Action::SetSelectedResult {
      result: SearchResultState {
        index: selected_result.index,
        path: selected_result.path.clone(),
        matches: selected_result.matches.clone(),
        total_matches: selected_result.total_matches,
      },
    });
    self.command_tx.as_ref().unwrap().send(action).unwrap();
  }

  fn calculate_total_matches(&mut self, search_result_state: &SearchResultState) -> &str {
    let total_matches: usize = search_result_state.matches.iter().map(|m| m.submatches.len()).sum();
    let total_matches_str = total_matches.to_string();
    self.match_counts.push(total_matches_str);
    self.match_counts.last().unwrap()
  }
}

impl Component for SearchResult {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if state.focused_screen == FocusedScreen::SearchResultList {
      match (key.code, key.modifiers) {
        (KeyCode::Char('d'), _) => {
          self.delete_file(state);
          Ok(None)
        },
        (KeyCode::Char('g') | KeyCode::Char('h') | KeyCode::Left, _) => {
          self.top(state);
          Ok(None)
        },
        (KeyCode::Char('G') | KeyCode::Char('l') | KeyCode::Right, _) => {
          self.bottom(state);
          Ok(None)
        },
        (KeyCode::Char('j') | KeyCode::Down, _) => {
          self.next(state);
          Ok(None)
        },
        (KeyCode::Char('k') | KeyCode::Up, _) => {
          self.previous(state);
          Ok(None)
        },
        (KeyCode::Enter, _) => {
          let action = AppAction::Action(Action::SetActiveTab { tab: Tab::Preview });
          self.command_tx.as_ref().unwrap().send(action).unwrap();
          Ok(None)
        },
        _ => Ok(None),
      }
    } else {
      Ok(None)
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
    let layout = get_layout(area);

    let block =
      Block::bordered().border_type(BorderType::Rounded).title(Title::from("Result List").alignment(Alignment::Left));
    let block = if state.focused_screen == FocusedScreen::SearchResultList {
      block.border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
      block
    };

    let project_root = state.project_root.to_string_lossy();
    let list_items: Vec<ListItem> = state
      .search_result
      .list
      .iter()
      .map(|s| {
        let text = Line::from(vec![
          // Display the relative path
          Span::raw(s.path.strip_prefix(format!("{}/", project_root).as_str()).unwrap_or(&s.path)),
          Span::raw(" ("),
          Span::styled(s.total_matches.to_string(), Style::default().fg(Color::Yellow)),
          Span::raw(")"),
        ]);
        ListItem::new(text)
      })
      .collect();

    let internal_selected = state.selected_result.index.unwrap_or(0);
    self.state.select(Some(internal_selected));
    self.set_selected_result(state);

    let details_widget = List::new(list_items)
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().bg(Color::Blue))
      .block(block);
    f.render_stateful_widget(details_widget, layout.search_details, &mut self.state);
    Ok(())
  }
}
