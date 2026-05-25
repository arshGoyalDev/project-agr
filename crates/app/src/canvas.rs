use crate::message::Message;

use iced::keyboard;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::event::{self, Event};
use iced::{Pixels, Point, Size};

use layout::DisplayList;
use layout::display_list::DrawCommand;

pub struct BrowserCanvas<'a> {
  pub display_list: &'a DisplayList,
  pub scroll_offset: f32,
  pub max_y: f32,
  pub url: String,
  pub active_tab_index: usize,
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
      Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
        let scroll_amount = match delta {
          mouse::ScrollDelta::Lines { y, .. } => y * 1000.0,
          mouse::ScrollDelta::Pixels { y, .. } => y * 10.0,
        };

        let max_scroll = (self.max_y - bounds.height).max(0.0);

        let new_offset = (self.scroll_offset - scroll_amount).clamp(0.0, max_scroll);

        (
          iced::widget::canvas::event::Status::Captured,
          Some(Message::ScrollChanged(new_offset)),
        )
      }
      Event::Keyboard(keyboard_event) => match keyboard_event {
        keyboard::Event::KeyPressed {
          key: keyboard::Key::Character(c),
          modifiers,
          ..
        } => {
          let char_str = c.as_str().to_lowercase();
          if modifiers.contains(keyboard::Modifiers::CTRL) && char_str == "w" {
            (event::Status::Captured, Some(Message::CloseTab(0, true)))
          } else if modifiers.contains(keyboard::Modifiers::CTRL) && char_str == "t" {
            (event::Status::Captured, Some(Message::NewTab))
          } else if modifiers.contains(keyboard::Modifiers::CTRL)
            && modifiers.contains(keyboard::Modifiers::SHIFT)
            && char_str == "r"
          {
            (
              event::Status::Captured,
              Some(Message::Reload(
                self.active_tab_index,
                self.url.clone(),
                None,
                true,
              )),
            )
          } else if modifiers.contains(keyboard::Modifiers::CTRL) && char_str == "r" {
            (
              event::Status::Captured,
              Some(Message::Reload(
                self.active_tab_index,
                self.url.clone(),
                None,
                false,
              )),
            )
          } else if modifiers.is_empty() {
            if let Some(ch) = c.chars().next() {
              (event::Status::Captured, Some(Message::KeyPressed(ch)))
            } else {
              (event::Status::Ignored, None)
            }
          } else {
            let new_offset = self.scroll_offset;
            (
              event::Status::Captured,
              Some(Message::ScrollChanged(new_offset)),
            )
          }
        }
        keyboard::Event::KeyPressed {
          key: keyboard::Key::Named(keyboard::key::Named::Enter),
          ..
        } => (event::Status::Captured, Some(Message::EnterPressed)),
        keyboard::Event::KeyPressed {
          key: keyboard::Key::Named(keyboard::key::Named::Backspace),
          ..
        } => (event::Status::Captured, Some(Message::BackspacePressed)),
        keyboard::Event::KeyPressed {
          key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
          ..
        } => {
          let total_content_height = self.max_y + 40.0;
          let scrollable_limit = (total_content_height - bounds.height).max(0.0);
          let new_offset = (self.scroll_offset + 20.0).min(scrollable_limit);

          (
            event::Status::Captured,
            Some(Message::ScrollChanged(new_offset)),
          )
        }
        keyboard::Event::KeyPressed {
          key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
          ..
        } => {
          let new_offset = (self.scroll_offset - 20.0).max(0.0);
          (
            event::Status::Captured,
            Some(Message::ScrollChanged(new_offset)),
          )
        }
        _ => {
          let new_offset = self.scroll_offset;
          (
            event::Status::Captured,
            Some(Message::ScrollChanged(new_offset)),
          )
        }
      },
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
        if let Some(cursor_position) = cursor.position_in(bounds) {
          return (
            event::Status::Captured,
            Some(Message::Click(cursor_position.x, cursor_position.y)),
          );
        }
        (event::Status::Ignored, Some(Message::TabBlur))
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
        DrawCommand::Circle(c) => {
          let screen_y = c.cy - self.scroll_offset;
          let circle = canvas::Path::circle(Point::new(c.cx, screen_y), c.radius);
          frame.fill(&circle, c.color);
        }
      }
    }

    if self.max_y > bounds.height {
      let view_ratio = bounds.height / self.max_y;
      let bar_height = (bounds.height * view_ratio).max(20.0);
      let scroll_ratio = self.scroll_offset / self.max_y;
      let bar_top = bounds.height * scroll_ratio;

      frame.fill_rectangle(
        iced::Point::new(bounds.width - 12.0, bar_top),
        iced::Size::new(10.0, bar_height),
        iced::Color::from_rgba8(100, 100, 100, 0.5),
      );
    }

    vec![frame.into_geometry()]
  }
}
