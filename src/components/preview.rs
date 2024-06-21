use std::{
  collections::HashMap,
  path::Path,
  process::{Command, Stdio},
  time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, symbols::scrollbar, widgets::*};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::AppAction,
  config::{Config, KeyBindings},
  layout::get_layout,
  redux::{
    action::Action,
    state::{ReplaceTextKind, SearchResultState, SearchTextKind, State, SubMatch},
    thunk::ThunkAction,
  },
  tabs::Tab,
};

#[derive(Default)]
pub struct Preview {
  command_tx: Option<UnboundedSender<AppAction>>,
  config: Config,
  lines_state: ListState,
  total_lines: usize,
  non_divider_lines: Vec<usize>,
}

impl Preview {
  pub fn new() -> Self {
    Self::default()
  }

  fn next(&mut self) {
    if let Some(current_index) = self.lines_state.selected() {
      let current_position = self.non_divider_lines.iter().position(|&index| index == current_index).unwrap_or(0);
      if current_position + 1 < self.non_divider_lines.len() {
        self.lines_state.select(Some(self.non_divider_lines[current_position + 1]));
      }
    } else if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(self.non_divider_lines[0]));
    }
  }

  fn previous(&mut self) {
    if let Some(current_index) = self.lines_state.selected() {
      let current_position = self.non_divider_lines.iter().position(|&index| index == current_index).unwrap_or(0);
      if current_position > 0 {
        self.lines_state.select(Some(self.non_divider_lines[current_position - 1]));
      }
    } else if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(self.non_divider_lines[0]));
    }
  }

  fn top(&mut self, state: &State) {
    if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(self.non_divider_lines[0]));
    }
  }

  fn bottom(&mut self, state: &State) {
    if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(self.non_divider_lines[self.non_divider_lines.len() - 1]));
    }
  }

  fn delete_line(&mut self, selected_result_state: &SearchResultState) {
    if let Some(selected_index) = self.lines_state.selected() {
      let line_index = self.non_divider_lines.iter().position(|&index| index == selected_index).unwrap_or(0);
      let file_index = selected_result_state.index.unwrap_or(0);
      let remove_line_from_file_thunk = AppAction::Thunk(ThunkAction::RemoveLineFromFile(file_index, line_index));
      self.command_tx.as_ref().unwrap().send(remove_line_from_file_thunk).unwrap();
    }
  }
}

impl Component for Preview {
  fn register_action_handler(&mut self, tx: UnboundedSender<AppAction>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, state: &State) -> Result<Option<AppAction>> {
    if state.active_tab == Tab::Preview {
      match (key.code, key.modifiers) {
        (KeyCode::Char('d'), _) => {
          self.delete_line(&state.selected_result);
          Ok(None)
        },
        (KeyCode::Char('g'), _) => {
          self.top(state);
          Ok(None)
        },
        (KeyCode::Char('G'), _) => {
          self.bottom(state);
          Ok(None)
        },

        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
          self.next();
          Ok(None)
        },
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
          self.previous();
          Ok(None)
        },
        (KeyCode::Enter, _) | (KeyCode::Esc, _) => {
          let action = AppAction::Action(Action::SetActiveTab { tab: Tab::SearchResult });
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
    let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Preview");
    let block =
      if state.active_tab == Tab::Preview { block.border_style(Style::default().fg(Color::Green)) } else { block };

    fn create_line_with_number<'a>(
      line_number: usize,
      line: &'a str,
      matches: &[SubMatch],
      replace_text: &'a str,
      search_kind: SearchTextKind,
      replace_kind: ReplaceTextKind,
    ) -> Line<'a> {
      let mut spans = vec![Span::styled(format!("{:4} ", line_number), Style::default().fg(Color::Blue))];
      let mut last_end = 0;

      for mat in matches {
        if mat.start > last_end {
          spans.push(Span::raw(&line[last_end..mat.start]));
        }

        let matched_text = &line[mat.start..mat.end];
        let replaced_text = match replace_kind {
          ReplaceTextKind::PreserveCase => {
            let first_char = matched_text.chars().next().unwrap_or_default();
            if matched_text.chars().all(char::is_uppercase) {
              replace_text.to_uppercase()
            } else if first_char.is_uppercase() {
              replace_text
                .chars()
                .enumerate()
                .map(|(i, rc)| if i == 0 { rc.to_uppercase().to_string() } else { rc.to_lowercase().to_string() })
                .collect::<String>()
            } else {
              replace_text.to_lowercase()
            }
          },
          ReplaceTextKind::Simple => replace_text.to_string(),
        };

        spans.push(Span::styled(matched_text, Style::default().fg(Color::Red).add_modifier(Modifier::CROSSED_OUT)));
        spans.push(Span::styled(replaced_text, Style::default().fg(Color::White).bg(Color::Green)));
        last_end = mat.end;
      }

      if last_end < line.len() {
        spans.push(Span::raw(&line[last_end..]));
      }
      Line::from(spans)
    }

    let mut lines = vec![];
    let mut last_match = None;
    self.non_divider_lines.clear();

    for result in &state.selected_result.matches {
      let line_number = result.line_number;
      let line = &result.lines.as_ref().unwrap().text;

      if let Some(last) = last_match {
        if line_number > last + 1 {
          let divider_line = Line::from("-".repeat(area.width as usize)).fg(Color::DarkGray);
          lines.push(divider_line.clone());
          lines.push(divider_line);
        }
      }

      self.non_divider_lines.push(lines.len());
      lines.push(create_line_with_number(
        line_number,
        line,
        &result.submatches,
        &state.replace_text.text,
        state.search_text.kind.clone(),
        state.replace_text.kind.clone(),
      ));

      last_match = Some(line_number);
    }

    self.total_lines = lines.len();
    let text = Text::from(lines);

    let preview_widget =
      List::new(text).highlight_style(Style::default().bg(Color::LightBlue)).block(block).scroll_padding(4);

    f.render_stateful_widget(preview_widget, layout.preview, &mut self.lines_state);

    Ok(())
  }
}
