use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Rect},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Text},
  widgets::{Block, BorderType, Clear, Padding, Paragraph, Widget, Wrap},
};

use crate::components::notifications::{self, NotificationEnum, NotificationWithTimestamp};

#[derive(Debug)]
pub struct NotificationBox<'a> {
  notification: &'a NotificationWithTimestamp,
  content: &'a String,
}
impl<'a> NotificationBox<'a> {
  pub fn new(notification: &'a NotificationWithTimestamp, content: &'a String) -> Self {
    Self { notification, content }
  }
}

impl<'a> Widget for NotificationBox<'a> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    Clear.render(area, buf);

    let color = match &self.notification.0 {
      NotificationEnum::Info(_) => Color::Blue,
      NotificationEnum::Warning(_) => Color::Yellow,
      NotificationEnum::Error(_) => Color::Red,
    };
    let block = Block::bordered()
      .border_type(BorderType::Rounded)
      .title("Notification")
      .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    let notification_text = Text::from(self.content.to_string())
      .style(Style::default().fg(color).bg(Color::Reset))
      .alignment(Alignment::Right);

    let notification = Paragraph::new(notification_text.clone()).wrap(Wrap { trim: true }).block(block.clone());

    notification.render(area, buf);
  }
}
