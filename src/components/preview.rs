use std::{
  collections::HashMap,
  path::Path,
  process::{Command, Stdio},
  time::Duration,
};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
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
    state::{FocusedScreen, ReplaceTextKind, SearchResultState, SearchTextKind, State, SubMatch},
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
    Self {
      command_tx: None,
      config: Config::default(),
      lines_state: {
        let mut state = ListState::default();
        state.select(Some(0));
        state
      },
      total_lines: 0,
      non_divider_lines: vec![],
    }
  }

  fn next(&mut self) {
    if let Some(current_index) = self.lines_state.selected() {
      let next_index = self
        .non_divider_lines
        .iter()
        .position(|&index| index > current_index)
        .map(|pos| self.non_divider_lines[pos])
        .unwrap_or_else(|| self.non_divider_lines[0]);
      self.lines_state.select(Some(next_index));
    } else if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(self.non_divider_lines[0]));
    }
  }

  fn previous(&mut self) {
    if let Some(current_index) = self.lines_state.selected() {
      let prev_index = self
        .non_divider_lines
        .iter()
        .rev()
        .position(|&index| index < current_index)
        .map(|pos| self.non_divider_lines[self.non_divider_lines.len() - 1 - pos])
        .unwrap_or_else(|| *self.non_divider_lines.last().unwrap());
      self.lines_state.select(Some(prev_index));
    } else if !self.non_divider_lines.is_empty() {
      self.lines_state.select(Some(*self.non_divider_lines.last().unwrap()));
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

  fn format_match_lines<'a>(
    &self,
    full_match: &'a str,
    submatches: &[SubMatch],
    replace_text: &'a String,
    replacement: &'a Option<String>,
    is_ast_grep: bool,
  ) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let match_lines: Vec<&str> = full_match.lines().collect();
    let replacement_lines: Vec<&str> = replacement.as_ref().map(|r| r.lines().collect()).unwrap_or_default();

    for (i, line) in match_lines.iter().enumerate() {
      let line_number = submatches[0].line_start + i;
      let mut spans = Vec::new();
      let mut last_end = 0;

      for submatch in submatches {
        if line_number >= submatch.line_start && line_number <= submatch.line_end {
          let start = if line_number == submatch.line_start { submatch.start } else { 0 };
          let end = if line_number == submatch.line_end { submatch.end } else { line.len() };

          if start > last_end {
            spans.push(Span::raw(&line[last_end..start]));
          }

          let matched_text = &line[start..end];
          if is_ast_grep {
            let replacement_line = replacement_lines.get(i).unwrap_or(&"");
            if replace_text.is_empty() {
              spans.push(Span::styled(matched_text, Style::default().bg(Color::Blue)));
            } else {
              let (common_prefix, common_suffix) = Self::find_common_parts(matched_text, replacement_line);

              spans.push(Span::raw(common_prefix));

              let search_diff = &matched_text[common_prefix.len()..matched_text.len() - common_suffix.len()];
              if !search_diff.trim().is_empty() {
                spans.push(Span::styled(
                  search_diff,
                  Style::default().fg(Color::White).bg(Color::LightRed).add_modifier(Modifier::CROSSED_OUT),
                ));
              }

              let replace_diff = &replacement_line[common_prefix.len()..replacement_line.len() - common_suffix.len()];
              if !replace_diff.trim().is_empty() {
                spans.push(Span::styled(replace_diff, Style::default().fg(Color::White).bg(Color::Green)));
              }

              spans.push(Span::raw(common_suffix));
            }
          } else if replace_text.is_empty() {
            spans.push(Span::styled(matched_text, Style::default().bg(Color::Blue)));
          } else {
            spans.push(Span::styled(
              matched_text,
              Style::default().fg(Color::LightRed).add_modifier(Modifier::CROSSED_OUT),
            ));
            spans.push(Span::styled(replace_text, Style::default().fg(Color::White).bg(Color::Green)));
          }

          last_end = end;
        }
      }

      if last_end < line.len() {
        spans.push(Span::raw(&line[last_end..]));
      }

      lines.push(Line::from(spans));
    }

    lines
  }

  fn find_common_parts<'a>(s1: &'a str, s2: &'a str) -> (&'a str, &'a str) {
    let mut prefix_len = 0;
    for (c1, c2) in s1.chars().zip(s2.chars()) {
      if c1 == c2 {
        prefix_len += 1;
      } else {
        break;
      }
    }

    let mut suffix_len = 0;
    for (c1, c2) in s1.chars().rev().zip(s2.chars().rev()) {
      if c1 == c2 && suffix_len < s1.len() - prefix_len && suffix_len < s2.len() - prefix_len {
        suffix_len += 1;
      } else {
        break;
      }
    }

    let common_prefix = &s1[..prefix_len];
    let common_suffix = &s1[s1.len() - suffix_len..];

    (common_prefix, common_suffix)
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
    if state.focused_screen == FocusedScreen::Preview {
      match (key.code, key.modifiers) {
        (KeyCode::Char('d'), _) => {
          self.delete_line(&state.selected_result);
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
          self.next();
          Ok(None)
        },
        (KeyCode::Char('k') | KeyCode::Up, _) => {
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

  fn update(&mut self, action: AppAction) -> Result<Option<AppAction>> {
    if let AppAction::Action(Action::SetSelectedResult { result }) = action {
      self.lines_state.select(Some(0));
    }

    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
    let layout = get_layout(area);
    let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Preview");
    let block = if state.focused_screen == FocusedScreen::Preview {
      block.border_style(Style::default().fg(Color::Green))
    } else {
      block
    };

    let mut lines = vec![];
    self.non_divider_lines.clear();

    for (match_index, result) in state.selected_result.matches.iter().enumerate() {
      let line_number = result.line_number;
      let start_index = lines.len();
      let is_selected = self.lines_state.selected().map(|s| s >= start_index).unwrap_or(false);

      for (i, line) in result.context_before.iter().enumerate() {
        let line_style = Style::default().fg(Color::DarkGray);
        let context_line_number = line_number.saturating_sub(result.context_before.len() - i);
        let spans = vec![
          Span::styled(format!("{:4} ", context_line_number), Style::default().fg(Color::Blue)),
          Span::styled(line, line_style),
        ];
        lines.push(Line::from(spans));
      }

      let is_ast_grep = matches!(state.search_text.kind, SearchTextKind::AstGrep);
      let formatted_lines = self.format_match_lines(
        &result.lines.as_ref().unwrap().text,
        &result.submatches,
        &state.replace_text.text,
        &result.replacement,
        is_ast_grep,
      );
      for (i, formatted_line) in formatted_lines.clone().into_iter().enumerate() {
        let mut spans = vec![Span::styled(format!("{:4} ", line_number + i), Style::default().fg(Color::LightGreen))];
        spans.extend(formatted_line.spans);
        self.non_divider_lines.push(lines.len());
        lines.push(Line::from(spans));
      }

      for (i, line) in result.context_after.iter().enumerate() {
        let line_style = Style::default().fg(Color::DarkGray);
        let spans = vec![
          Span::styled(format!("{:4} ", line_number + formatted_lines.len() + i), Style::default().fg(Color::Blue)),
          Span::styled(line, line_style),
        ];
        lines.push(Line::from(spans));
      }

      let divider_color = if is_selected { Color::Yellow } else { Color::DarkGray };
      lines.push(Line::from("-".repeat(area.width as usize)).fg(divider_color));
    }

    self.total_lines = lines.len();
    let text = Text::from(lines);

    let highlight_style = Style::default().add_modifier(Modifier::BOLD).fg(Color::White);

    let preview_widget =
      List::new(text).highlight_style(highlight_style).block(block).highlight_symbol("> ").scroll_padding(4);

    f.render_stateful_widget(preview_widget, layout.preview, &mut self.lines_state);

    Ok(())
  }
}
