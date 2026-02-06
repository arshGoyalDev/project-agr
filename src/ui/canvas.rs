use iced::widget::canvas;
use iced::{Color, Point, Size};

use crate::app::Message;
use crate::rendering::DisplayList;

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
    _cursor: iced::mouse::Cursor,
  ) -> (canvas::event::Status, Option<Message>) {
    match event {
      canvas::Event::Mouse(iced::mouse::Event::WheelScrolled {delta}) => {
        match delta {
          iced::mouse::ScrollDelta::Lines { y, .. } |
          iced::mouse::ScrollDelta::Pixels { y, .. } => {
            let total_content_height = self.max_y + 40.0;
            let scrollable_limit = (total_content_height - bounds.height).max(0.0);

            let target_offset = self.scroll_offset - (y * 20.0);
            let clamped_offset = target_offset.max(0.0).min(scrollable_limit);

            (
              canvas::event::Status::Captured,
              Some(Message::ScrollChanged(clamped_offset))
            )
          }
        }
      }
      canvas::Event::Keyboard(iced::keyboard::Event::KeyPressed {key, ..}) => {
        let mut new_offset = 0.0;
        match key {
          iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
            new_offset = (self.scroll_offset - 20.0).max(0.0)

          }
          iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
            let total_content_height = self.max_y + 40.0;
            let scrollable_limit = (total_content_height - bounds.height).max(0.0);
            let target_offset = self.scroll_offset + 20.0;
            new_offset = target_offset.min(scrollable_limit);
          }
          _ => ()
        }
        (
          canvas::event::Status::Captured,
          Some(Message::ScrollChanged(new_offset))
        )
      }
      _ => (canvas::event::Status::Ignored, None),
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

    for item in self.display_list.items() {
      let screen_y = item.y - self.scroll_offset;

      if screen_y >= -20.0 && screen_y <= bounds.height + 20.0 {
        frame.fill_text(canvas::Text {
          content: item.character.to_string(),
          position: iced::Point::new(item.x, screen_y),
          ..Default::default()
        });
      }
    }

    if self.max_y > 0.0 {
      let view_ratio = self.height / self.max_y;
      let bar_height = self.height * view_ratio;

      let scroll_ratio = self.scroll_offset / self.max_y;
      let bar_top = self.height * scroll_ratio;

      if bar_top.is_finite() && bar_height.is_finite() {
        let rectangle = canvas::Path::rectangle(
          Point::new(bounds.width - 10.0, bar_top), 
          Size::new(10.0, bar_height)
        );
        
        frame.fill(&rectangle, Color::BLACK);
      }
    }

    vec![frame.into_geometry()]
  }
}
