use crate::browser::Browser;
use crate::canvas::BrowserCanvas;
use crate::dom::get_page_bg_color;
use crate::message::Message;
use crate::window_controls::window_controls;

use iced::widget::{
  Button, Canvas, Column, Container, MouseArea, Row, Space, Text, TextInput, button,
};
use iced::{Background, Border, Color, Element, Length, Shadow};

impl Browser {
  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = &self.tabs[self.active_tab_index];

    let can_go_back = active_tab.history_index > 0;
    let can_go_forward = active_tab.history_index + 1 < active_tab.history.len();

    // Tab Bar
    let mut tab_row = Row::new().spacing(4).align_y(iced::Alignment::Center);

    for (i, tab) in self.tabs.iter().enumerate() {
      let is_active = i == self.active_tab_index;
      let is_hovered = Some(i) == self.hovered_tab;

      let raw_title = if !tab.title.is_empty() {
        tab.title.clone()
      } else if !tab.url.is_empty() {
        tab.url.clone()
      } else {
        "New Tab".to_string()
      };

      let display_title = if raw_title.len() > 20 {
        format!("{}...", &raw_title[..17])
      } else {
        raw_title
      };

      let label_btn = Button::new(
        Text::new(if is_active {
          format!("{}", display_title)
        } else {
          display_title
        })
        .size(14.0),
      )
      .on_press(Message::SwitchTab(i))
      .style(button::text);

      let mut single_tab_content = Row::new().align_y(iced::Alignment::Center).push(label_btn);

      if is_active || is_hovered {
        let close_btn = Button::new(Text::new("×").size(14.0))
          .on_press(Message::CloseTab(i))
          .style(button::text)
          .padding([0, 4]);
        single_tab_content = single_tab_content.push(close_btn);
      } else {
        single_tab_content = single_tab_content.push(Space::with_width(Length::Fixed(18.0)));
      }

      let tab_mouse_area = MouseArea::new(single_tab_content)
        .on_enter(Message::TabHovered(i))
        .on_exit(Message::TabUnhovered);

      let tab_container = Container::new(tab_mouse_area)
        .padding([0.0, 8.0])
        .height(Length::Fixed(32.0))
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| {
          if is_active {
            iced::widget::container::Style {
              background: Some(Color::from_rgba8(50, 50, 50, 1.0).into()),
              border: Border {
                radius: 4.0.into(),
                ..Default::default()
              },
              ..Default::default()
            }
          } else {
            iced::widget::container::Style::default()
          }
        });

      tab_row = tab_row.push(tab_container);
    }

    // New Tab Button
    tab_row = tab_row.push(
      Button::new(
        Container::new(Text::new("+").size(14.0))
          .width(Length::Fill)
          .height(Length::Fill)
          .align_x(iced::alignment::Horizontal::Center)
          .align_y(iced::alignment::Vertical::Center),
      )
      .on_press(Message::NewTab)
      .style(|_theme, status| button::Style {
        background: Some(Background::Color(Color::from_rgba8(
          50,
          50,
          50,
          match status {
            button::Status::Pressed | button::Status::Hovered => 1.0,
            _ => 0.0,
          },
        ))),
        text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.85),
        border: Border {
          radius: 4.0.into(),
          width: 0.0,
          color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
      })
      .padding(0)
      .width(Length::Fixed(32.0))
      .height(Length::Fixed(32.0)),
    );

    // Title Bar wrapper
    let title_bar_content = Row::new()
      .align_y(iced::Alignment::Center)
      .push(tab_row)
      .push(Space::with_width(Length::Fill))
      .push(window_controls(
        Message::MinimizeWindow,
        Message::ToggleMaximizeWindow,
        Message::CloseWindow,
      ));

    let draggable_title_bar = MouseArea::new(title_bar_content).on_press(Message::TitleBarPressed);

    let title_bar = Container::new(draggable_title_bar)
      .width(Length::Fill)
      .padding(8)
      .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb8(30, 30, 30).into()),
        ..Default::default()
      });

    // Address Bar
    let mut back_btn = Button::new(Text::new("<")).style(button::text);
    if can_go_back {
      back_btn = back_btn.on_press(Message::GoBack);
    }

    let mut forward_btn = Button::new(Text::new(">")).style(button::text);
    if can_go_forward {
      forward_btn = forward_btn.on_press(Message::GoForward);
    }

    let is_bookmarked = self.bookmarks.contains(&active_tab.url);
    let bookmark_btn = Button::new(
      Text::new(if is_bookmarked { "★" } else { "☆" })
        .size(18.0)
        .color(if is_bookmarked {
          Color::from_rgb8(255, 215, 0)
        } else {
          Color::WHITE
        }),
    )
    .style(button::text)
    .on_press(Message::ToggleBookmark);

    let address_bar = Row::new()
      .spacing(10)
      .padding(5)
      .push(back_btn)
      .push(forward_btn)
      .push(
        TextInput::new("Enter URL...", &self.address_bar_text)
          .on_input(Message::AddressInputChanged)
          .on_submit(Message::NavigateTo(self.address_bar_text.clone())),
      )
      .push(bookmark_btn);

    // Canvas rendering
    let browser_canvas = BrowserCanvas {
      display_list: &active_tab.display_list,
      scroll_offset: active_tab.scroll_offset,
      max_y: active_tab.max_y,
    };

    let content = Canvas::new(browser_canvas)
      .width(Length::Fill)
      .height(Length::Fill);

    let mut canvas_bg_color = iced::Color::WHITE;
    if let Some(tree) = &active_tab.tree {
      if let Some(extracted_color) = get_page_bg_color(tree) {
        canvas_bg_color = extracted_color;
      }
    }

    // Final Combine
    Column::new()
      .push(title_bar)
      .push(address_bar)
      .push(
        Container::new(content)
          .width(Length::Fill)
          .height(Length::Fill)
          .style(move |_theme| iced::widget::container::Style {
            background: Some(canvas_bg_color.into()),
            ..Default::default()
          }),
      )
      .into()
  }
}
