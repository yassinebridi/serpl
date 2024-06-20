use std::cmp::min;

use ratatui::{prelude::*, widgets::*};

const VERTICAL_CONSTRAINTS: [Constraint; 3] = [
  Constraint::Length(3), // Search input height
  Constraint::Length(3), // Replace input height
  Constraint::Min(0),    // Remaining space for search details
];

const HORIZONTAL_CONSTRAINTS: [Constraint; 2] = [
  Constraint::Percentage(30), // Left column
  Constraint::Percentage(70), // Right column
];

const STATUS_HORIZONTAL_CONSTRAINTS: [Constraint; 2] = [
  Constraint::Percentage(70), // status/help note
  Constraint::Percentage(30), // Status/spinning loader
];

const STATUS_CONSTRAINT: Constraint = Constraint::Length(1);

pub struct LayoutRects {
  pub search_input: Rect,
  pub replace_input: Rect,
  pub search_details: Rect,
  pub status_left: Rect,
  pub status_right: Rect,
  pub preview: Rect,
}

pub fn get_layout(area: Rect) -> LayoutRects {
  // Split the area into main content and status
  let main_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), STATUS_CONSTRAINT].as_ref())
    .split(area);

  // Split the main content into left and right columns
  let horizontal_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(HORIZONTAL_CONSTRAINTS.as_ref())
    .split(main_layout[0]);

  // Split the left column into vertical sections
  let vertical_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(VERTICAL_CONSTRAINTS.as_ref())
    .split(horizontal_layout[0]);

  // Split the status area into left and right parts
  let status_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(STATUS_HORIZONTAL_CONSTRAINTS.as_ref())
    .split(main_layout[1]);

  LayoutRects {
    search_input: vertical_layout[0],
    replace_input: vertical_layout[1],
    search_details: vertical_layout[2],
    preview: horizontal_layout[1],
    status_left: status_layout[0],
    status_right: status_layout[1],
  }
}

pub fn get_notification_layout(rect: Rect, content: &str, i: u16) -> Rect {
  let line_width = content.lines().map(|line| line.len()).max().unwrap_or(0) as u16 + 4;
  let line_height = content.lines().count() as u16 + 2;
  let right = rect.width - line_width - 1;
  let bottom = rect.height - line_height - i * 3 - 2;

  Rect::new(right, bottom, line_width, line_height)
}
