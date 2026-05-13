use crate::message::Message;

use iced::keyboard;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::event::{self, Event};
use iced::{Color, Pixels, Point, Size};

use layout::DisplayList;
use layout::display_list::DrawCommand;

pub struct BrowserCanvas<'a> {
  pub display_list: &'a DisplayList,
  pub scroll_offset: f32,
  pub max_y: f32,
  pub height: f32,
}

impl<'a> canvas::Program<Message> for BrowserCanvas<'a> {
  type State = ();

  fn update(
    &self,
    _state: &mut Self::State,
    event: canvas::Event,
    bounds: iced::Rectangle,
    cursor: iced::mouse::Cursor,
  ) -> (event::Status, Option<Message>) {
    match event {
      Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
        mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
          let total_content_height = self.max_y + 40.0;
          let scrollable_limit = (total_content_height - bounds.height).max(0.0);
          let clamped_offset = (self.scroll_offset - y * 20.0)
            .max(0.0)
            .min(scrollable_limit);
          (
            event::Status::Captured,
            Some(Message::ScrollChanged(clamped_offset)),
          )
        }
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
        let new_offset = match key {
          keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
            (self.scroll_offset - 20.0).max(0.0)
          }
          keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
            let total_content_height = self.max_y + 40.0;
            let scrollable_limit = (total_content_height - bounds.height).max(0.0);
            (self.scroll_offset + 20.0).min(scrollable_limit)
          }
          _ => self.scroll_offset,
        };
        (
          event::Status::Captured,
          Some(Message::ScrollChanged(new_offset)),
        )
      }
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
        if let Some(cursor_position) = cursor.position_in(bounds) {
          return (
            event::Status::Captured,
            Some(Message::Click(cursor_position.x, cursor_position.y)),
          );
        }
        (event::Status::Ignored, None)
      }
      _ => (event::Status::Ignored, None),
    }
  }

  fn draw(
    &self,
    _state: &Self::State,
    renderer: &iced::Renderer,
    _theme: &iced::Theme,
    bounds: iced::Rectangle,
    _cursor: iced::mouse::Cursor,
  ) -> Vec<canvas::Geometry> {
    let mut frame = canvas::Frame::new(renderer, bounds.size());

    for cmd in self.display_list.items() {
      let top = cmd.top() - self.scroll_offset;
      let bottom = cmd.bottom() - self.scroll_offset;
      if bottom < -20.0 || top > bounds.height + 20.0 {
        continue;
      }

      match cmd {
        DrawCommand::Text(t) => {
          let screen_y = t.y - self.scroll_offset;
          frame.fill_text(canvas::Text {
            content: t.word.clone(),
            position: Point::new(t.x, screen_y),
            color: t.color,
            font: t.font,
            size: Pixels(t.size),
            ..Default::default()
          });
        }
        DrawCommand::Rect(r) => {
          let screen_y = r.y1 - self.scroll_offset;
          let rect = canvas::Path::rectangle(
            Point::new(r.x1, screen_y),
            Size::new(r.x2 - r.x1, r.y2 - r.y1),
          );
          frame.fill(&rect, r.color);
        }
      }
    }

    if self.max_y > 0.0 {
      let view_ratio = self.height / self.max_y;
      let bar_height = self.height * view_ratio;
      let scroll_ratio = self.scroll_offset / self.max_y;
      let bar_top = self.height * scroll_ratio;

      if bar_top.is_finite() && bar_height.is_finite() {
        let scrollbar = canvas::Path::rectangle(
          Point::new(bounds.width - 10.0, bar_top),
          Size::new(10.0, bar_height),
        );
        frame.fill(&scrollbar, Color::BLACK);
      }
    }

    vec![frame.into_geometry()]
  }
}
