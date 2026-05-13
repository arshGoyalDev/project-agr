use crate::tab::Tab;
use crate::window_controls::window_controls;

use iced::widget::{
  Button, Canvas, Column, Container, MouseArea, Row, Space, Text, TextInput, button,
};
use iced::{Background, Border, Color, Element, Length, Shadow, Subscription, Task, window};

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
  pub hovered_tab: Option<usize>, // NEW: Track mouse hover for the 'x' button
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
        hovered_tab: None, // Initialize hover state
        address_bar_text: url.clone(),
        width: 800.0,
        height: 600.0,
      },
      Task::done(Message::LoadUrl(0, url)),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height))
  }

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

      // NEW: Hover and Close states for Tabs
      Message::TabHovered(index) => {
        self.hovered_tab = Some(index);
        Task::none()
      }
      Message::TabUnhovered => {
        self.hovered_tab = None;
        Task::none()
      }
      Message::CloseTab(index) => {
        if self.tabs.len() > 1 {
          self.tabs.remove(index);

          // Adjust active tab index if necessary
          if self.active_tab_index >= index && self.active_tab_index > 0 {
            self.active_tab_index -= 1;
          } else if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
          }

          // Update address bar to reflect the new active tab
          self.address_bar_text = self.tabs[self.active_tab_index].url.clone();
          Task::none()
        } else {
          // If closing the last tab, replace it with a blank one
          self.tabs[0] = Tab::new("about:blank".to_string());
          self.address_bar_text = "about:blank".to_string();
          Task::done(Message::LoadUrl(0, "about:blank".to_string()))
        }
      }

      Message::NewTab => {
        self.tabs.push(Tab::new("about:blank".to_string()));
        self.active_tab_index = self.tabs.len() - 1;
        self.address_bar_text = "about:blank".to_string();
        Task::done(Message::LoadUrl(self.active_tab_index, "about:blank".to_string()))
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

        Task::done(Message::LoadUrl(self.active_tab_index, url))
      }
      Message::GoBack => {
        let tab = self.active_tab_mut();

        if tab.history_index > 0 {
          tab.history_index -= 1;
          let prev_url = tab.history[tab.history_index].clone();
          return Task::done(Message::LoadUrl(self.active_tab_index, prev_url));
        }

        Task::none()
      }
      Message::GoForward => {
        let tab = self.active_tab_mut();

        if tab.history_index + 1 < tab.history.len() {
          tab.history_index += 1;
          let next_url = tab.history[tab.history_index].clone();
          return Task::done(Message::LoadUrl(self.active_tab_index, next_url));
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
              url_handler.init(self.active_tab().url.clone(), false);
              let resolved_url = url_handler.resolve(&href);
              return Task::done(Message::NavigateTo(resolved_url));
            }
          }
        }
        Task::none()
      }
      Message::LoadUrl(tab_index, url) => {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
          tab.url = url.clone();
          if tab_index == self.active_tab_index {
            self.address_bar_text = url.clone();
          }
          tab.title = String::from("Loading...");

          // Clear the canvas visually while loading
          tab.display_list = layout::DisplayList::new();
        }

        // Send the network request to the background!
        Task::perform(
          fetch_html_task(url),
          move |(base_url, is_view_source, result)| {
            Message::HtmlFetched(tab_index, base_url, is_view_source, result)
          },
        )
      }

      Message::HtmlFetched(tab_index, base_url, is_view_source, result) => {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
          if let Ok(body) = result {
            let mut html_parser = HTMLParser::new(body);
            let mut tree = html_parser.parse();

            // Handle view-source
            if is_view_source {
              let highlighted = syntax_highlight(&tree);
              tree = HTMLParser::new(highlighted).parse();
            }

            let mut links = Vec::new();
            find_stylesheet_links(&tree, &mut links);

            tab.tree = Some(tree);

            if links.is_empty() {
              // No CSS to fetch, immediately go to the next step
              return Task::done(Message::CssFetched(tab_index, vec![]));
            } else {
              // Fetch CSS in the background!
              return Task::perform(fetch_css_task(links, base_url), move |bodies| {
                Message::CssFetched(tab_index, bodies)
              });
            }
          } else {
            tab.title = String::from("Network Error");
          }
        }
        Task::none()
      }

      Message::CssFetched(tab_index, css_bodies) => {
        let width = self.width;
        if let Some(tab) = self.tabs.get_mut(tab_index) {
          if let Some(tree) = &tab.tree {
            let default_css = include_str!("../../../browser.css").to_string();
            let mut css_parser = CSSParser::new(&default_css);
            let mut rules = css_parser.parse();

            // 1. Add downloaded CSS
            for body in css_bodies {
              let mut linked_parser = CSSParser::new(&body);
              rules.extend(linked_parser.parse());
            }

            // 2. Add inline styles
            let mut inline_style_texts = Vec::new();
            find_inline_styles(tree, &mut inline_style_texts);
            for css_text in inline_style_texts {
              rules.extend(CSSParser::new(&css_text).parse());
            }

            rules.sort_by_key(|r| r.priority);
            style(tree, &rules);

            // Set title
            if let Some(title) = extract_title(tree) {
              tab.title = title;
            } else {
              tab.title = tab.url.clone();
            }

            // Layout & Paint!
            let mut doc = DocumentLayout::new(tree);
            doc.layout(width);

            tab.display_list = doc.paint();
            tab.max_y = tab.display_list.max_y();
            tab.document = Some(doc);
            tab.scroll_offset = 0.0;
          }
        }
        Task::none()
      }
      // Message::LoadUrl(url) => {
      //   let width = self.width;

      //   self.address_bar_text = url.clone();

      //   {
      //     let tab = self.active_tab_mut();
      //     tab.url = url.clone();
      //     tab.title = String::new(); // Reset title while loading
      //   }

      //   let mut url_handler = URLHandler::default();
      //   url_handler.init(url.clone(), false);
      //   let body_result = url_handler.request();

      //   let mut new_tree = None;
      //   if let Ok(value) = body_result {
      //     let mut html_parser = HTMLParser::new(value);
      //     new_tree = Some(html_parser.parse());
      //   }

      //   // View Source support
      //   if url_handler.view_source {
      //     if let Some(node) = &new_tree {
      //       let highlighted = syntax_highlight(node);
      //       let mut html_parser = HTMLParser::new(highlighted);
      //       new_tree = Some(html_parser.parse());
      //     }
      //   }

      //   // CSS Styling
      //   if let Some(node) = &new_tree {
      //     let default_css = include_str!("../../../browser.css").to_string();
      //     let mut css_parser = CSSParser::new(&default_css);
      //     let mut rules = css_parser.parse();

      //     let mut links = Vec::new();
      //     find_stylesheet_links(node, &mut links);

      //     for link in links {
      //       let resolved_url = url_handler.resolve(&link);
      //       let mut style_handler = URLHandler::default();
      //       style_handler.init(resolved_url, false);

      //       if let Ok(css_body) = style_handler.request() {
      //         let mut linked_parser = CSSParser::new(&css_body);
      //         rules.extend(linked_parser.parse());
      //       }
      //     }

      //     let mut inline_style_texts = Vec::new();
      //     find_inline_styles(node, &mut inline_style_texts);
      //     for css_text in inline_style_texts {
      //       rules.extend(CSSParser::new(&css_text).parse());
      //     }

      //     rules.sort_by_key(|r| r.priority);
      //     style(node, &rules);
      //   }

      //   let tab = self.active_tab_mut();
      //   tab.tree = new_tree;

      //   if let Some(node) = &tab.tree {
      //     // NEW: Extract Title from the HTML Tree
      //     if let Some(title) = extract_title(node) {
      //       tab.title = title;
      //     }

      //     let mut doc = DocumentLayout::new(node);
      //     doc.layout(width);

      //     tab.display_list = doc.paint();
      //     tab.max_y = tab.display_list.max_y();
      //     tab.document = Some(doc);
      //   }

      //   tab.scroll_offset = 0.0;
      //   Task::none()
      // }
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

    // Tab Bar (Left side)
    let mut tab_row = Row::new().spacing(4).align_y(iced::Alignment::Center);

    for (i, tab) in self.tabs.iter().enumerate() {
      let is_active = i == self.active_tab_index;
      let is_hovered = Some(i) == self.hovered_tab;

      // Determine display title (Fallback to URL -> "New Tab")
      let raw_title = if !tab.title.is_empty() {
        tab.title.clone()
      } else if !tab.url.is_empty() {
        tab.url.clone()
      } else {
        "New Tab".to_string()
      };

      // Truncate long titles to keep the tabs readable
      let display_title = if raw_title.len() > 20 {
        format!("{}...", &raw_title[..17])
      } else {
        raw_title
      };

      // Construct the Tab Button label
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

      // Render the Close 'X' button conditionally
      if is_active || is_hovered {
        let close_btn = Button::new(Text::new("×").size(14.0))
          .on_press(Message::CloseTab(i))
          .style(button::text)
          .padding([0, 4]); // Tight padding for the X
        single_tab_content = single_tab_content.push(close_btn);
      } else {
        // Invisible spacer so the tab width doesn't shift wildly when hovered
        single_tab_content = single_tab_content.push(Space::with_width(Length::Fixed(18.0)));
      }

      // Wrap tab content in a MouseArea for hover detection
      let tab_mouse_area = MouseArea::new(single_tab_content)
        .on_enter(Message::TabHovered(i))
        .on_exit(Message::TabUnhovered);

      // Wrap in a Container to highlight the active tab
      let tab_container = Container::new(tab_mouse_area)
        .padding([0.0, 8.0]) // CHANGED: Removed vertical padding so Fixed height takes over
        .height(Length::Fixed(32.0)) // NEW: Lock height to 32px
        .align_y(iced::alignment::Vertical::Center) // NEW: Keep text centered
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
            button::Status::Pressed => 1.0,
            button::Status::Hovered => 1.0,
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

    // Build Window Controls (Right side)
    let window_controls = window_controls(
      Message::MinimizeWindow,
      Message::ToggleMaximizeWindow,
      Message::CloseWindow,
    );

    // Combine Tabs, a flexible Space, and Window Controls
    let title_bar_content = Row::new()
      .align_y(iced::Alignment::Center)
      .push(tab_row)
      .push(Space::with_width(Length::Fill))
      .push(window_controls);

    // Wrap in MouseArea to detect dragging
    let draggable_title_bar = MouseArea::new(title_bar_content).on_press(Message::TitleBarPressed);

    // Wrap the MouseArea in the dark Container
    let title_bar = Container::new(draggable_title_bar)
      .width(Length::Fill)
      .padding(8)
      .style(|_theme| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb8(30, 30, 30).into()),
        ..Default::default()
      });

    // Address Bar Buttons
    let mut back_btn = Button::new(Text::new("<")).style(button::text);
    if can_go_back {
      back_btn = back_btn.on_press(Message::GoBack);
    }

    let mut forward_btn = Button::new(Text::new(">")).style(button::text);
    if can_go_forward {
      forward_btn = forward_btn.on_press(Message::GoForward);
    }

    // Address Bar
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

    // Web Content Canvas
    let browser_canvas = BrowserCanvas {
      display_list: &active_tab.display_list,
      scroll_offset: active_tab.scroll_offset,
      max_y: active_tab.max_y,
      height: self.height,
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

    // Final layout
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

// NEW FUNCTION: Extract the <title> tag from the HTML Tree
pub fn extract_title(node_rc: &Rc<RefCell<Node>>) -> Option<String> {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "title" {
      for child_rc in &e.children {
        if let Node::Text(t) = &*child_rc.borrow() {
          let trimmed = t.text.trim();
          if !trimmed.is_empty() {
            return Some(trimmed.to_string());
          }
        }
      }
    }
  }

  for child in node.children() {
    if let Some(title) = extract_title(child) {
      return Some(title);
    }
  }

  None
}

pub fn get_page_bg_color(node_rc: &Rc<RefCell<Node>>) -> Option<iced::Color> {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "html" || e.tag == "body" {
      // Access the style map for this node
      if let Some(bgcolor) = node.style().get("background-color") {
        if bgcolor != "transparent" {
          if let Some(color) = parse_css_color(bgcolor) {
            return Some(color);
          }
        }
      }
    }
  }

  for child in node.children() {
    if let Some(color) = get_page_bg_color(child) {
      return Some(color);
    }
  }

  None
}

pub fn parse_css_color(s: &str) -> Option<iced::Color> {
  let s = s.trim();

  match s {
    "black" => return Some(iced::Color::BLACK),
    "white" => return Some(iced::Color::WHITE),
    "red" => return Some(iced::Color::from_rgb(1.0, 0.0, 0.0)),
    "green" => return Some(iced::Color::from_rgb(0.0, 0.502, 0.0)),
    "blue" => return Some(iced::Color::from_rgb(0.0, 0.0, 1.0)),
    "lightblue" => return Some(iced::Color::from_rgb(0.678, 0.847, 0.902)),
    "gray" | "grey" => return Some(iced::Color::from_rgb(0.502, 0.502, 0.502)),
    "yellow" => return Some(iced::Color::from_rgb(1.0, 1.0, 0.0)),
    "orange" => return Some(iced::Color::from_rgb(1.0, 0.647, 0.0)),
    "purple" => return Some(iced::Color::from_rgb(0.502, 0.0, 0.502)),
    "transparent" => return None,
    _ => {}
  }

  if s.starts_with('#') && s.len() == 7 {
    let r = u8::from_str_radix(&s[1..3], 16).ok()?;
    let g = u8::from_str_radix(&s[3..5], 16).ok()?;
    let b = u8::from_str_radix(&s[5..7], 16).ok()?;
    return Some(iced::Color::from_rgb(
      r as f32 / 255.0,
      g as f32 / 255.0,
      b as f32 / 255.0,
    ));
  }

  if s.starts_with('#') && s.len() == 4 {
    let r = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
    let g = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
    let b = u8::from_str_radix(&s[3..4].repeat(2), 16).ok()?;
    return Some(iced::Color::from_rgb(
      r as f32 / 255.0,
      g as f32 / 255.0,
      b as f32 / 255.0,
    ));
  }
  None
}

// Runs in the background to fetch the main HTML
pub async fn fetch_html_task(url: String) -> (String, bool, Result<String, String>) {
  let mut handler = net::URLHandler::default();
  handler.init(url.clone(), false);

  match handler.request() {
    Ok(body) => (url, handler.view_source, Ok(body)),
    Err(_) => (url, handler.view_source, Err("Network Error".to_string())),
  }
}

// Runs in the background to fetch all CSS stylesheets
pub async fn fetch_css_task(links: Vec<String>, base_url: String) -> Vec<String> {
  let mut css_bodies = Vec::new();

  for link in links {
    let mut url_handler = net::URLHandler::default();
    url_handler.init(base_url.clone(), false);
    let resolved_url = url_handler.resolve(&link);

    let mut style_handler = net::URLHandler::default();
    style_handler.init(resolved_url, false);

    if let Ok(css_body) = style_handler.request() {
      css_bodies.push(css_body);
    }
  }
  css_bodies
}
