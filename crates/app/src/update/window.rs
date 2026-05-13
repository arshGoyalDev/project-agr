use crate::browser::Browser;
use crate::message::Message;
use html_parser::Node;
use iced::{Task, window};
use net::URLHandler;

pub fn title_bar_pressed() -> Task<Message> {
  window::get_oldest().and_then(window::drag)
}

pub fn minimize_window() -> Task<Message> {
  window::get_oldest().and_then(|id| window::minimize(id, true))
}

pub fn toggle_maximize_window() -> Task<Message> {
  window::get_oldest().and_then(window::toggle_maximize)
}

pub fn close_window() -> Task<Message> {
  iced::exit()
}

pub fn window_resized(browser: &mut Browser, width: f32, height: f32) -> Task<Message> {
  browser.width = width;
  browser.height = height;
  browser.relayout();
  Task::none()
}

pub fn scroll_changed(browser: &mut Browser, offset: f32) -> Task<Message> {
  browser.tabs[browser.active_tab_index].scroll_offset = offset;
  Task::none()
}

pub fn address_input_changed(browser: &mut Browser, text: String) -> Task<Message> {
  browser.address_bar_text = text;
  Task::none()
}

pub fn click(browser: &mut Browser, x: f32, y: f32) -> Task<Message> {
  let offset = browser.tabs[browser.active_tab_index].scroll_offset;
  let abs_y = y + offset;

  let active_tab = &browser.tabs[browser.active_tab_index];

  if let Some(doc) = &active_tab.document {
    if let Some(mut current_node) = doc.get_node(x, abs_y) {
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
        println!("Clicked link: {}", href);

        if href.starts_with('#') {
          return Task::done(Message::NavigateTo(href));
        }

        let mut url_handler = URLHandler::default();
        url_handler.init(active_tab.url.clone(), false);
        let resolved = url_handler.resolve(&href);
        return Task::done(Message::NavigateTo(resolved));
      }
    }
  }

  Task::none()
}
