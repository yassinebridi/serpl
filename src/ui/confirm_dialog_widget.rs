use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Text},
  widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{redux::state::DialogAction, utils::centered_rect_with_size};

#[derive(Default, Debug, Clone)]
pub struct ConfirmDialogWidget {
  pub title: String,
  pub message: String,
  pub confirm_label: String,
  pub cancel_label: String,
  pub show_cancel: bool,
}

#[derive(Default, Debug, Clone)]
pub enum ConfirmDialogAction {
  Confirm,
  #[default]
  Cancel,
}

#[derive(Default, Debug, Clone)]
pub struct ConfirmDialogState {
  pub selected_button: ConfirmDialogAction,
}

impl ConfirmDialogState {
  pub fn new() -> Self {
    Self { selected_button: ConfirmDialogAction::Cancel }
  }

  pub fn loop_selected_button(&mut self) {
    self.selected_button = match self.selected_button {
      ConfirmDialogAction::Confirm => ConfirmDialogAction::Cancel,
      ConfirmDialogAction::Cancel => ConfirmDialogAction::Confirm,
    };
  }

  pub fn set_selected_button(&mut self, action: ConfirmDialogAction) {
    self.selected_button = action;
  }
}

impl ConfirmDialogWidget {
  pub fn new(title: String, message: String, confirm_label: String, cancel_label: String, show_cancel: bool) -> Self {
    Self { title, message, confirm_label, cancel_label, show_cancel }
  }
}

impl StatefulWidget for ConfirmDialogWidget {
  type State = ConfirmDialogState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let horizontal_padding = 2u16;
    let vertical_padding = 2u16;
    let buttons_padding = 2u16;

    let block = Block::default()
      .title(self.title)
      .title_alignment(Alignment::Center)
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(Style::default().fg(Color::Yellow));

    let confirm_button_style = match state.selected_button {
      ConfirmDialogAction::Confirm => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
      ConfirmDialogAction::Cancel => Style::default().fg(Color::White),
    };
    let cancel_button_style = match state.selected_button {
      ConfirmDialogAction::Confirm => Style::default().fg(Color::White),
      ConfirmDialogAction::Cancel => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    };
    let confirm_button = Paragraph::new(self.confirm_label.to_string())
      .style(confirm_button_style)
      .alignment(Alignment::Center)
      .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Yellow)));
    let cancel_button = Paragraph::new(self.cancel_label.to_string())
      .style(cancel_button_style)
      .alignment(Alignment::Center)
      .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Yellow)));

    let confirm_button_size = (self.confirm_label.len() + buttons_padding as usize) as u16;
    let cancel_button_size = (self.cancel_label.len() + buttons_padding as usize) as u16;

    let text = self.message;

    let width = text.lines().map(|line| line.len()).max().unwrap_or(0) as u16 + 4;
    let height = text.lines().count() as u16 + 6;

    let lines = Text::from(text);
    let text_widget = Paragraph::new(lines)
      .block(Block::new().padding(Padding::new(
        horizontal_padding,
        horizontal_padding,
        vertical_padding,
        vertical_padding,
      )))
      .alignment(Alignment::Center)
      .style(Style::new().white())
      .wrap(Wrap { trim: true });

    let centered_area = centered_rect_with_size(width, height, area);

    let main_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(1), Constraint::Max(2)])
      .split(centered_area);

    Clear.render(centered_area, buf);
    text_widget.render(main_layout[0], buf);
    block.render(centered_area, buf);

    let buttons_layout = Layout::default().direction(Direction::Horizontal);
    let c = (main_layout[1].width - (confirm_button_size + cancel_button_size)) / 2; // 19
    let buttons_layout = buttons_layout
      .constraints([
        Constraint::Length(c),
        Constraint::Max(confirm_button_size),
        Constraint::Max(cancel_button_size),
        Constraint::Length(c),
      ])
      .split(main_layout[1]);
    confirm_button.render(buttons_layout[1], buf);
    cancel_button.render(buttons_layout[2], buf);
  }
}
