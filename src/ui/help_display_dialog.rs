use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Text},
  widgets::{block::Title, Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{redux::state::DialogAction, utils::centered_rect_with_size};

#[derive(Default, Debug, Clone)]
pub struct HelpDisplayDialogWidget {
  pub tabs: Vec<Tab>,
  pub active_tab: usize,
}
#[derive(Default, Debug, Clone)]
pub struct Tab {
  pub title: String,
  pub content: String,
}

#[derive(Default, Debug, Clone)]
pub struct HelpDisplayDialogState {
  pub show: bool,
}

impl HelpDisplayDialogState {
  pub fn new() -> Self {
    Self { show: false }
  }
}

impl HelpDisplayDialogWidget {
  pub fn new(tabs: Vec<Tab>, active_tab: usize) -> Self {
    Self { tabs, active_tab }
  }
}

impl StatefulWidget for HelpDisplayDialogWidget {
  type State = HelpDisplayDialogState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let horizontal_padding = 2u16;
    let vertical_padding = 2u16;
    let buttons_padding = 2u16;

    let block = Block::default()
      .title(Line::from("Press 'q' to close").alignment(Alignment::Right))
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(Style::default().fg(Color::Yellow));

    let block_with_tabs = self.tabs.iter().enumerate().fold(block, |acc_block, (index, tab)| {
      let title_style = if index == self.active_tab { Style::default().fg(Color::Green) } else { Style::default() };
      acc_block.title(Line::from(format!(" {} ", tab.title)).style(title_style))
    });

    let text = self.tabs[self.active_tab].content.clone();

    let width = 80;
    let height = 10;

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
    block_with_tabs.render(centered_area, buf);

    let buttons_layout = Layout::default().direction(Direction::Horizontal);
    let c = main_layout[1].width;
    let buttons_layout =
      buttons_layout.constraints([Constraint::Length(c), Constraint::Length(c)]).split(main_layout[1]);
  }
}
