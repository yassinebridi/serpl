use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Text},
  widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{redux::state::DialogAction, utils::centered_rect_with_size};

#[derive(Default, Debug, Clone)]
pub struct DisplayDialogWidget {
  pub title: String,
  pub message: String,
}

#[derive(Default, Debug, Clone)]
pub struct DisplayDialogState {
  pub show: bool,
}

impl DisplayDialogState {
  pub fn new() -> Self {
    Self { show: false }
  }
}

impl DisplayDialogWidget {
  pub fn new(title: String, message: String) -> Self {
    Self { title, message }
  }
}

impl StatefulWidget for DisplayDialogWidget {
  type State = DisplayDialogState;

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
      .alignment(Alignment::Left)
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
    let c = main_layout[1].width;
    let buttons_layout =
      buttons_layout.constraints([Constraint::Length(c), Constraint::Length(c)]).split(main_layout[1]);
  }
}
