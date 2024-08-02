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
use tui_input::{backend::crossterm::EventHandler, Input};

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

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct SearchResult {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  state: ListState,
  match_counts: Vec<String>,
  search_input: Input,
  is_searching: bool,
  search_matches: Vec<usize>,
  current_match_index: usize,
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

  fn replace_single_file(&mut self, state: &State) {
    if let Some(selected_index) = self.state.selected() {
      if selected_index < state.search_result.list.len() {
        let process_single_file_replace_thunk = AppAction::Thunk(ThunkAction::ProcessSingleFileReplace(selected_index));
        self.command_tx.as_ref().unwrap().send(process_single_file_replace_thunk).unwrap();
      }
    }
  }

  fn handle_local_key_events(&mut self, key: KeyEvent, state: &State) {
    match (key.code, key.modifiers) {
      (KeyCode::Char('d'), _) => {
        self.delete_file(state);
      },
      (KeyCode::Char('g') | KeyCode::Char('h') | KeyCode::Left, _) => {
        self.top(state);
      },
      (KeyCode::Char('G') | KeyCode::Char('l') | KeyCode::Right, _) => {
        self.bottom(state);
      },
      (KeyCode::Char('j') | KeyCode::Down, _) => {
        self.next(state);
      },
      (KeyCode::Char('k') | KeyCode::Up, _) => {
        self.previous(state);
      },
      (KeyCode::Char('r'), _) => {
        self.replace_single_file(state);
      },
      (KeyCode::Esc, _) => {
        self.is_searching = false;
        self.search_matches.clear();
        self.search_input.reset();
        self.current_match_index = 0;
      },
      (KeyCode::Enter, _) => {
        let action = AppAction::Action(Action::SetActiveTab { tab: Tab::Preview });
        self.command_tx.as_ref().unwrap().send(action).unwrap();
      },
      (KeyCode::Char('n'), _) => {
        self.next_match(state);
      },
      (KeyCode::Char('p'), _) => {
        self.previous_match(state);
      },
      _ => {},
    }
  }

  fn handle_search_input(&mut self, key: KeyEvent, state: &State) {
    match key.code {
      KeyCode::Esc | KeyCode::Enter => {
        self.is_searching = false;
      },
      _ => {
        self.search_input.handle_event(&crossterm::event::Event::Key(key));
        self.perform_search(state);
      },
    }
  }

  fn perform_search(&mut self, state: &State) {
    let search_term = self.search_input.value().to_lowercase();
    self.search_matches.clear();
    self.current_match_index = 0;

    for (index, result) in state.search_result.list.iter().enumerate() {
      if result.path.to_lowercase().contains(&search_term) {
        let result_index = result.index.unwrap();
        self.search_matches.push(result_index);
      }
    }
    log::info!("111Search matches: {:?}", self.search_matches);

    if !self.search_matches.is_empty() {
      self.state.select(Some(self.search_matches[0]));
      self.update_selected_result(state);
    }
  }

  fn next_match(&mut self, state: &State) {
    log::info!("Next match");
    log::info!("Search matches: {:?}", self.search_matches);
    if !self.search_matches.is_empty() {
      self.current_match_index = (self.current_match_index + 1) % self.search_matches.len();
      let next_index = self.search_matches[self.current_match_index];
      self.state.select(Some(next_index));
      self.update_selected_result(state);
    }
  }

  fn previous_match(&mut self, state: &State) {
    if !self.search_matches.is_empty() {
      self.current_match_index = (self.current_match_index + self.search_matches.len() - 1) % self.search_matches.len();
      let prev_index = self.search_matches[self.current_match_index];
      self.state.select(Some(prev_index));
      self.update_selected_result(state);
    }
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
      match key.code {
        KeyCode::Char('/') => {
          self.is_searching = true;
          self.search_input.reset();
          Ok(None)
        },
        _ if self.is_searching => {
          self.handle_search_input(key, state);
          Ok(None)
        },
        _ => {
          self.handle_local_key_events(key, state);
          Ok(None)
        },
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
    let results_to_display = &state.search_result.list;
    let search_term = self.search_input.value().to_lowercase();

    let list_items: Vec<ListItem> = results_to_display
      .iter()
      .enumerate()
      .map(|(index, s)| {
        let path = s.path.strip_prefix(format!("{}/", project_root).as_str()).unwrap_or(&s.path);
        let mut spans = Vec::new();
        let mut start = 0;

        if !search_term.is_empty() {
          for (idx, _) in path.to_lowercase().match_indices(&search_term) {
            if start < idx {
              spans.push(Span::raw(&path[start..idx]));
            }
            spans.push(Span::styled(
              &path[idx..idx + search_term.len()],
              Style::default().bg(Color::Yellow).fg(Color::Black),
            ));
            start = idx + search_term.len();
          }
        }

        if start < path.len() {
          spans.push(Span::raw(&path[start..]));
        }

        spans.push(Span::raw(" ("));
        spans.push(Span::styled(s.total_matches.to_string(), Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(")"));

        ListItem::new(Line::from(spans))
      })
      .collect();

    let internal_selected = self.state.selected().unwrap_or(0);

    let details_widget = List::new(list_items)
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().bg(Color::Blue))
      .block(block);
    f.render_stateful_widget(details_widget, layout.search_details, &mut self.state);

    if self.is_searching {
      let search_input = Paragraph::new(self.search_input.value())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Search"));
      let input_area = Rect::new(
        layout.search_details.x,
        layout.search_details.y + layout.search_details.height - 3,
        layout.search_details.width,
        3,
      );
      f.render_widget(search_input, input_area);
      f.set_cursor(input_area.x + self.search_input.cursor() as u16 + 1, input_area.y + 1);
    }

    Ok(())
  }
}
