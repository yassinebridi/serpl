use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Rect},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Text},
  widgets::{Block, BorderType, Clear, Padding, Paragraph, Widget, Wrap},
};

#[derive(Default, Debug)]
pub struct SmallHelpWidget {
  content: String,
  color: Color,
  alignment: Alignment,
}
impl SmallHelpWidget {
  pub fn new(content: String, color: Color, alignment: Alignment) -> Self {
    Self { content, color, alignment }
  }
}

impl Widget for SmallHelpWidget {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let small_help_text = Text::from(self.content).style(Style::default().fg(self.color).bg(Color::Reset));

    let small_help = Paragraph::new(small_help_text).wrap(Wrap { trim: true }).alignment(self.alignment);
    // .block(Block::default().padding(if self.alignment == Alignment::Left { Padding::left(1) } else { Padding::right(1) }));
    small_help.render(area, buf);
  }
}
