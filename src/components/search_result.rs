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
    state::{SearchResultState, State},
    thunk::ThunkAction,
  },
  tabs::Tab,
};

#[derive(Default)]
pub struct SearchResult {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  state: ListState,
  match_counts: Vec<String>, // Store the string representations of match counts
}

impl SearchResult {
  pub fn new() -> Self {
    Self::default()
  }

  fn next(&mut self, state: &State) {
    if state.search_result.list.is_empty() {
      return;
    }

    let i = match self.state.selected() {
      Some(i) => {
        if i >= state.search_result.list.len() - 1 {
          0
        } else {
          i + 1
        }
      },
      None => 0,
    };
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

  fn previous(&mut self, state: &State) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          state.search_result.list.len() - 1
        } else {
          i - 1
        }
      },
      None => 0,
    };
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

  fn delete_file(&mut self, selected_result_state: &SearchResultState) {
    if let Some(selected_index) = selected_result_state.index {
      let remove_file_from_list_thunk = AppAction::Thunk(ThunkAction::RemoveFileFromList(selected_index));
      self.command_tx.as_ref().unwrap().send(remove_file_from_list_thunk).unwrap();
      self.state.select(Some(selected_index));
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
    if state.active_tab == Tab::SearchResult {
      match key.code {
        KeyCode::Char('d') => {
          self.delete_file(&state.selected_result);
          Ok(None)
        },
        KeyCode::Char('j') => {
          self.next(state);
          Ok(None)
        },
        KeyCode::Char('k') => {
          self.previous(state);
          Ok(None)
        },
        KeyCode::Enter => {
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
      Block::bordered().border_type(BorderType::Rounded).title(Title::from("Details").alignment(Alignment::Left));
    let block = if state.active_tab == Tab::SearchResult {
      block.border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
      block
    };

    let list_items: Vec<ListItem> = state
      .search_result
      .list
      .iter()
      .map(|s| {
        let text = Line::from(vec![
          Span::raw(&s.path),
          Span::raw(" ("),
          Span::styled(s.total_matches.to_string(), Style::default().fg(Color::Yellow)),
          Span::raw(")"),
        ]);
        ListItem::new(text)
      })
      .collect();

    let details_widget = List::new(list_items)
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().bg(Color::LightBlue))
      .block(block);
    f.render_stateful_widget(details_widget, layout.search_details, &mut self.state);
    Ok(())
  }
}
