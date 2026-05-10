use iced::widget::{Button, Container, Row, Text, button};
use iced::{Background, Border, Color, Length, Shadow, Theme};

fn close_style(_theme: &Theme, status: button::Status) -> button::Style {
  match status {
    button::Status::Hovered | button::Status::Pressed => button::Style {
      background: Some(Background::Color(Color::from_rgba(
        0.75,
        0.18,
        0.18,
        if matches!(status, button::Status::Pressed) {
          0.7
        } else {
          1.0
        },
      ))),
      text_color: Color::WHITE,
      border: Border {
        radius: 4.0.into(),
        width: 0.0,
        color: Color::TRANSPARENT,
      },
      shadow: Shadow::default(),
    },
    _ => button::Style {
      background: None,
      text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.45),
      border: Border {
        radius: 4.0.into(),
        width: 0.0,
        color: Color::TRANSPARENT,
      },
      shadow: Shadow::default(),
    },
  }
}

fn neutral_style(_theme: &Theme, status: button::Status) -> button::Style {
  match status {
    button::Status::Hovered | button::Status::Pressed => button::Style {
      background: Some(Background::Color(Color::from_rgba(
        1.0,
        1.0,
        1.0,
        if matches!(status, button::Status::Pressed) {
          0.07
        } else {
          0.10
        },
      ))),
      text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.85),
      border: Border {
        radius: 4.0.into(),
        width: 0.0,
        color: Color::TRANSPARENT,
      },
      shadow: Shadow::default(),
    },
    _ => button::Style {
      background: None,
      text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.45),
      border: Border {
        radius: 4.0.into(),
        width: 0.0,
        color: Color::TRANSPARENT,
      },
      shadow: Shadow::default(),
    },
  }
}

fn ctrl_btn<'a, Message: Clone + 'a>(
  label: &'a str,
  font_size: f32,
  msg: Message,
  style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
) -> Button<'a, Message> {
  Button::new(
    Container::new(Text::new(label).size(font_size))
      .width(Length::Fill)
      .height(Length::Fill)
      .align_x(iced::alignment::Horizontal::Center)
      .align_y(iced::alignment::Vertical::Center),
  )
  .on_press(msg)
  .style(style)
  .padding(0)
  .width(Length::Fixed(32.0))
  .height(Length::Fixed(30.0))
}

pub fn window_controls<Message>(
  on_minimize: Message,
  on_maximize: Message,
  on_close: Message,
) -> Row<'static, Message>
where
  Message: Clone + 'static,
{
  Row::new()
    .spacing(0)
    .align_y(iced::Alignment::Center)
    .push(ctrl_btn("–", 14.0, on_minimize, neutral_style))
    .push(ctrl_btn("□", 11.0, on_maximize, neutral_style))
    .push(ctrl_btn("×", 16.0, on_close, close_style))
}
