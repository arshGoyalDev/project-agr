use crate::tab::Tab;

use iced::widget::{
  Button, Canvas, Column, Container, MouseArea, Row, Space, Text, TextInput, button,
};
use iced::{Element, Length, Subscription, Task, window};

use css_parser::{CSSParser, style};
use html_parser::{HTMLParser, Node};
use layout::{DocumentLayout, syntax_highlight};
use net::URLHandler;
use ui::{BrowserCanvas, Message};

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

pub struct Browser {
  pub tabs: Vec<Tab>,
  pub active_tab_index: usize,
  pub address_bar_text: String,
  pub width: f32,
  pub height: f32,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("about:blank");
    let args: Vec<String> = env::args().collect();

    if let Some(value) = args.get(1) {
      url = value.to_string();
    }

    let initial_tab = Tab::new(url.clone());

    (
      Self {
        tabs: vec![initial_tab],
        active_tab_index: 0,
        address_bar_text: url.clone(),
        width: 800.0,
        height: 600.0,
      },
      Task::done(Message::LoadUrl(url)),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height))
  }

  // Helper functions to grab the active tab easily
  fn active_tab_mut(&mut self) -> &mut Tab {
    &mut self.tabs[self.active_tab_index]
  }

  fn active_tab(&self) -> &Tab {
    &self.tabs[self.active_tab_index]
  }

  fn relayout(&mut self) {
    let width = self.width;
    let tab = self.active_tab_mut();

    if let Some(doc) = &mut tab.document {
      doc.layout(width);
      tab.display_list = doc.paint();
      tab.max_y = tab.display_list.max_y();
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::TitleBarPressed => window::get_oldest().and_then(window::drag),
      Message::MinimizeWindow => window::get_oldest().and_then(|id| window::minimize(id, true)),
      Message::ToggleMaximizeWindow => window::get_oldest().and_then(window::toggle_maximize),
      Message::CloseWindow => iced::exit(),
      Message::NewTab => {
        self.tabs.push(Tab::new("about:blank".to_string()));
        self.active_tab_index = self.tabs.len() - 1;
        self.address_bar_text = "about:blank".to_string();
        Task::done(Message::LoadUrl("about:blank".to_string()))
      }
      Message::SwitchTab(index) => {
        if index < self.tabs.len() {
          self.active_tab_index = index;
          self.address_bar_text = self.tabs[index].url.clone();
        }
        Task::none()
      }
      Message::NavigateTo(url) => {
        let tab = self.active_tab_mut();

        tab.history.truncate(tab.history_index + 1);

        if tab.history.last() != Some(&url) {
          tab.history.push(url.clone());
          tab.history_index = tab.history.len() - 1;
        }

        Task::done(Message::LoadUrl(url))
      }
      Message::GoBack => {
        let tab = self.active_tab_mut();

        println!("GO BACK!!");
        if tab.history_index > 0 {
          println!("GO BACK TWICE!!");
          tab.history_index -= 1;
          let prev_url = tab.history[tab.history_index].clone();
          return Task::done(Message::LoadUrl(prev_url));
        }

        Task::none()
      }
      Message::GoForward => {
        let tab = self.active_tab_mut();

        if tab.history_index + 1 < tab.history.len() {
          tab.history_index += 1;
          let next_url = tab.history[tab.history_index].clone();
          return Task::done(Message::LoadUrl(next_url));
        }

        Task::none()
      }
      Message::AddressInputChanged(text) => {
        self.address_bar_text = text;
        Task::none()
      }
      Message::ScrollChanged(offset) => {
        self.active_tab_mut().scroll_offset = offset;
        Task::none()
      }
      Message::Click(x, y) => {
        let offset = self.active_tab().scroll_offset;
        let abs_x = x;
        let abs_y = y + offset;

        if let Some(doc) = &self.active_tab().document {
          // Using get_node based on your previous code upload
          if let Some(mut current_node) = doc.get_node(abs_x, abs_y) {
            let mut clicked_href = None;

            loop {
              let parent_opt = {
                let node_borrow = current_node.borrow();
                if let Node::Element(e) = &*node_borrow {
                  if e.tag == "a" {
                    if let Some(href) = e.attributes.get("href") {
                      clicked_href = Some(href.clone());
                    }
                  }
                }

                match &*node_borrow {
                  Node::Element(e) => e.parent.as_ref().and_then(|w| w.upgrade()),
                  Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
                }
              };

              if clicked_href.is_some() {
                break;
              }

              match parent_opt {
                Some(parent) => current_node = parent,
                None => break,
              }
            }

            if let Some(href) = clicked_href {
              println!("Clicked link! Redirecting to: {}", href);
              let mut url_handler = URLHandler::default();
              // Resolve the URL relative to the CURRENT tab's URL
              url_handler.init(self.active_tab().url.clone(), false);
              let resolved_url = url_handler.resolve(&href);
              return Task::done(Message::NavigateTo(resolved_url));
            }
          }
        }
        Task::none()
      }
      Message::LoadUrl(url) => {
        let width = self.width;

        {
          let tab = self.active_tab_mut();
          tab.url = url.clone();
          self.address_bar_text = url.clone();
        }

        let mut url_handler = URLHandler::default();
        url_handler.init(url.clone(), false);
        let body_result = url_handler.request();

        let mut new_tree = None;
        if let Ok(value) = body_result {
          let mut html_parser = HTMLParser::new(value);
          new_tree = Some(html_parser.parse());
        }

        // View Source support
        if url_handler.view_source {
          if let Some(node) = &new_tree {
            let highlighted = syntax_highlight(node);
            let mut html_parser = HTMLParser::new(highlighted);
            new_tree = Some(html_parser.parse());
          }
        }

        // CSS Styling
        if let Some(node) = &new_tree {
          let default_css = include_str!("../../../browser.css").to_string();
          let mut css_parser = CSSParser::new(&default_css);
          let mut rules = css_parser.parse();

          let mut links = Vec::new();
          find_stylesheet_links(node, &mut links);

          for link in links {
            let resolved_url = url_handler.resolve(&link);
            let mut style_handler = URLHandler::default();
            style_handler.init(resolved_url, false);

            if let Ok(css_body) = style_handler.request() {
              let mut linked_parser = CSSParser::new(&css_body);
              rules.extend(linked_parser.parse());
            }
          }

          let mut inline_style_texts = Vec::new();
          find_inline_styles(node, &mut inline_style_texts);
          for css_text in inline_style_texts {
            rules.extend(CSSParser::new(&css_text).parse());
          }

          rules.sort_by_key(|r| r.priority);
          style(node, &rules);
        }

        // 3. Layout and assign back to the Active Tab
        let tab = self.active_tab_mut();
        tab.tree = new_tree;

        if let Some(node) = &tab.tree {
          let mut doc = DocumentLayout::new(node);
          doc.layout(width);

          tab.display_list = doc.paint();
          tab.max_y = tab.display_list.max_y();
          tab.document = Some(doc);
        }

        tab.scroll_offset = 0.0;
        Task::none()
      }
      Message::WindowResized(width, height) => {
        self.width = width;
        self.height = height;
        self.relayout();
        Task::none()
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = self.active_tab();

    let can_go_back = active_tab.history_index > 0;
    let can_go_forward = active_tab.history_index + 1 < active_tab.history.len();

    // 1. Build Tab Bar (Left side)
    let mut tab_row = Row::new().spacing(2).align_y(iced::Alignment::Center);
    for (i, tab) in self.tabs.iter().enumerate() {
      let label = if i == self.active_tab_index {
        format!(
          "{}",
          if tab.url.is_empty() {
            "New Tab"
          } else {
            &tab.url
          }
        )
      } else {
        if tab.url.is_empty() {
          "New Tab".to_string()
        } else {
          tab.url.clone()
        }
      };

      tab_row = tab_row.push(
        Button::new(Text::new(label))
          .on_press(Message::SwitchTab(i))
          .style(button::text),
      );
    }
    tab_row = tab_row.push(
      Button::new(Text::new("+"))
        .on_press(Message::NewTab)
        .style(button::text),
    );

    // NEW: Build Window Controls (Right side)
    let window_controls = Row::new()
      .spacing(5)
      .align_y(iced::Alignment::Center)
      .push(
        Button::new(Text::new("_"))
          .on_press(Message::MinimizeWindow)
          .style(button::text).align_y(iced::Alignment::Center),
      )
      .push(
        Button::new(Text::new("□"))
          .on_press(Message::ToggleMaximizeWindow)
          .style(button::text),
      )
      .push(
        Button::new(Text::new("X"))
          .on_press(Message::CloseWindow)
          .style(button::text),
      );

    // NEW: Combine Tabs, a flexible Space, and Window Controls
    let title_bar_content = Row::new()
      .align_y(iced::Alignment::Center)
      .push(tab_row)
      .push(Space::with_width(Length::Fill)) // Pushes controls to the far right!
      .push(window_controls);

    // NEW: Wrap in MouseArea to detect dragging
    let draggable_title_bar = MouseArea::new(title_bar_content).on_press(Message::TitleBarPressed);

    // UPDATED: Wrap the MouseArea in the dark Container
    let title_bar = Container::new(draggable_title_bar)
      .width(Length::Fill)
      .padding(8)
      .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb8(30, 30, 30).into()),
        ..Default::default()
      });

    // 2. Build Address Bar Buttons
    let mut back_btn = Button::new(Text::new("<")).style(button::text);
    if can_go_back {
      back_btn = back_btn.on_press(Message::GoBack);
    }

    let mut forward_btn = Button::new(Text::new(">")).style(button::text);
    if can_go_forward {
      forward_btn = forward_btn.on_press(Message::GoForward);
    }

    // 3. Build Address Bar
    let address_bar = Row::new()
      .spacing(10)
      .padding(5)
      .push(back_btn)
      .push(forward_btn)
      .push(
        TextInput::new("Enter URL...", &self.address_bar_text)
          .on_input(Message::AddressInputChanged)
          .on_submit(Message::NavigateTo(self.address_bar_text.clone())),
      );

    // 4. Build Web Content Canvas
    let browser_canvas = BrowserCanvas {
      display_list: &active_tab.display_list,
      scroll_offset: active_tab.scroll_offset,
      max_y: active_tab.max_y,
      height: self.height,
    };

    let content = Canvas::new(browser_canvas)
      .width(Length::Fill)
      .height(Length::Fill);

    // 5. Combine into final layout
    Column::new()
      .push(title_bar)
      .push(address_bar)
      .push(
        Container::new(content)
          .width(Length::Fill)
          .height(Length::Fill),
      )
      .into()
  }

  pub fn theme(&self) -> iced::Theme {
    iced::Theme::Dark
  }
}

pub fn find_stylesheet_links(node_rc: &Rc<RefCell<Node>>, links: &mut Vec<String>) {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "link" {
      if let Some(rel) = e.attributes.get("rel") {
        if rel == "stylesheet" {
          if let Some(href) = e.attributes.get("href") {
            links.push(href.clone());
          }
        }
      }
    }
  }

  for child in node.children() {
    find_stylesheet_links(child, links);
  }
}

pub fn find_inline_styles(node_rc: &Rc<RefCell<Node>>, inline_rules: &mut Vec<String>) {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "style" {
      for child_rc in &e.children {
        let child = child_rc.borrow();
        if let Node::Text(t) = &*child {
          inline_rules.push(t.text.clone());
        }
      }
    }
  }

  for child in node.children() {
    find_inline_styles(child, inline_rules);
  }
}
