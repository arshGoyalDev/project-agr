use iced::{Task, window};

use crate::browser::Browser;
use crate::message::Message;

use html_parser::Node;
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

pub fn key_pressed(browser: &mut Browser, c: char) -> Task<Message> {
  browser.tabs[browser.active_tab_index].keypress(c);
  browser.relayout();
  Task::none()
}

pub fn backspace_pressed(browser: &mut Browser) -> Task<Message> {
  browser.tabs[browser.active_tab_index].backspace();
  browser.relayout();
  Task::none()
}

pub fn click(browser: &mut Browser, x: f32, y: f32) -> Task<Message> {
  let offset = browser.tabs[browser.active_tab_index].scroll_offset;
  let abs_y = y + offset;

  let mut clicked_href = None;
  let mut clicked_input = None;
  let mut form_submission = None;
  let mut relayout_needed = false;

  {
    let active_tab = &mut browser.tabs[browser.active_tab_index];

    if let Some(doc) = &active_tab.document {
      if let Some(mut current_node) = doc.get_node(x, abs_y) {
        if active_tab.focus.is_some() {
          active_tab.focus = None;
          relayout_needed = true;
        }

        let mut clicked_button = None;

        loop {
          let parent_opt = {
            let mut node_borrow = current_node.borrow_mut();
            if let Node::Element(e) = &mut *node_borrow {
              if e.tag == "a" {
                if let Some(href) = e.attributes.get("href") {
                  clicked_href = Some(href.clone());
                }
              } else if e.tag == "input" {
                e.attributes.insert("value".to_string(), "".to_string());
                clicked_input = Some(current_node.clone());
              } else if e.tag == "button" {
                clicked_button = Some(current_node.clone());
              }
            }

            match &*node_borrow {
              Node::Element(e) => e.parent.as_ref().and_then(|w| w.upgrade()),
              Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
            }
          };

          if clicked_href.is_some() || clicked_input.is_some() || clicked_button.is_some() {
            break;
          }

          match parent_opt {
            Some(parent) => current_node = parent,
            None => break,
          }
        }

        if let Some(button_node) = clicked_button {
          form_submission = active_tab.submit_form(button_node);
        }
      } else {
        // Clicked empty space — just clear focus
        if active_tab.focus.is_some() {
          active_tab.focus = None;
          relayout_needed = true;
        }
      }
    }
  }

  if let Some(href) = clicked_href {
    println!("Clicked link: {}", href);
    if href.starts_with('#') {
      return Task::done(Message::NavigateTo(href));
    }
    let mut url_handler = URLHandler::default();
    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);
    let resolved = url_handler.resolve(&href);
    return Task::done(Message::NavigateTo(resolved));
  } else if let Some(input_node) = clicked_input {
    browser.tabs[browser.active_tab_index].focus = Some(input_node);
    browser.relayout();
    return Task::none();
  } else if let Some((action, payload)) = form_submission {
    let mut url_handler = URLHandler::default();
    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);
    let resolved = url_handler.resolve(&action);

    let final_url = if resolved.contains('?') {
      format!("{}&{}", resolved, payload)
    } else {
      format!("{}?{}", resolved, payload)
    };
    return Task::done(Message::NavigateTo(final_url));
  } else if let Some((action, payload)) = form_submission {
    let mut url_handler = URLHandler::default();
    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);
    let resolved = url_handler.resolve(&action);

    return Task::done(Message::LoadUrl(
      browser.active_tab_index,
      resolved,
      Some(payload),
    ));
  }

  if relayout_needed {
    browser.relayout();
  }

  Task::none()
}
