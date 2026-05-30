use iced::keyboard::Key;
use iced::keyboard::key::Named;

use iced::{Task, window};

use crate::browser::Browser;
use crate::message::Message;

use html_parser::Node;
use net::URLHandler;

use std::cell::RefCell;
use std::rc::Rc;

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
  if browser.tabs[browser.active_tab_index].blur() {
    browser.relayout();
  }
  Task::none()
}

pub fn blink_cursor(browser: &mut Browser) -> Task<Message> {
  browser.cursor_blink_visible = !browser.cursor_blink_visible;
  let visible_str = if browser.cursor_blink_visible {
    "true"
  } else {
    "false"
  };

  let mut needs_repaint = false;
  {
    let tab = &mut browser.tabs[browser.active_tab_index];
    if let Some(focus_node) = &tab.focus {
      let mut borrow = focus_node.borrow_mut();
      if let Node::Element(e) = &mut *borrow {
        e.attributes
          .insert("data-cursor-visible".to_string(), visible_str.to_string());
        needs_repaint = true;
      }
    }
  }

  if needs_repaint {
    let tab = &mut browser.tabs[browser.active_tab_index];
    if let Some(doc) = &mut tab.document {
      tab.display_list = doc.paint();
    }
  }

  Task::none()
}

pub fn tab_blur(browser: &mut Browser) -> Task<Message> {
  if browser.tabs[browser.active_tab_index].blur() {
    browser.relayout();
  }

  Task::none()
}

pub fn key_pressed(browser: &mut Browser, key: Key) -> Task<Message> {
  let mut needs_relayout = false;

  if browser.tabs[browser.active_tab_index].focus.is_some() {
    let tab = &mut browser.tabs[browser.active_tab_index];

    match key {
      Key::Character(c) => {
        for ch in c.as_str().chars() {
          tab.keypress(ch);
        }
        needs_relayout = true;
      }
      Key::Named(Named::Backspace) => {
        tab.backspace();
        needs_relayout = true;
      }
      Key::Named(Named::Delete) => {
        tab.delete();
        needs_relayout = true;
      }
      Key::Named(Named::ArrowLeft) => {
        tab.arrow_left();
        needs_relayout = true;
      }
      Key::Named(Named::ArrowRight) => {
        tab.arrow_right();
        needs_relayout = true;
      }
      Key::Named(Named::Enter) => {
        return enter_pressed(browser);
      }
      _ => {}
    }

    if needs_relayout {
      browser.cursor_blink_visible = true;
      if let Some(focus_node) = &browser.tabs[browser.active_tab_index].focus {
        let mut borrow = focus_node.borrow_mut();
        if let html_parser::Node::Element(e) = &mut *borrow {
          e.attributes
            .insert("data-cursor-visible".to_string(), "true".to_string());
        }
      }
      browser.relayout();
    }
  }

  Task::none()
}

fn enter_pressed(browser: &mut Browser) -> Task<Message> {
  let mut form_submission = None;

  {
    let active_tab = &mut browser.tabs[browser.active_tab_index];

    if let Some(focus_node) = &active_tab.focus {
      form_submission = active_tab.submit_form(focus_node.clone());
    }
  }

  if let Some((action, payload, method)) = form_submission {
    let mut url_handler = URLHandler::default();

    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);

    let resolved = url_handler.resolve(&action);

    if method == "GET" {
      let final_url = if payload.is_empty() {
        resolved
      } else if resolved.contains('?') {
        format!("{}&{}", resolved, payload)
      } else {
        format!("{}?{}", resolved, payload)
      };

      return Task::done(Message::NavigateTo(final_url, None));
    } else {
      return Task::done(Message::NavigateTo(resolved, Some(payload)));
    }
  }

  Task::none()
}

fn clear_all_radios(node_rc: &Rc<RefCell<Node>>, target_name: &str) {
  {
    let mut node = node_rc.borrow_mut();
    if let Node::Element(e) = &mut *node {
      if e.tag == "input"
        && e.attributes.get("type").map(|s| s.trim().to_lowercase()) == Some("radio".to_string())
      {
        if e.attributes.get("name").map(|s| s.as_str()) == Some(target_name) {
          e.attributes.remove("checked");
        }
      }
    }
  }

  let children = {
    let node = node_rc.borrow();
    node.children().iter().map(Rc::clone).collect::<Vec<_>>()
  };

  for child in children {
    clear_all_radios(&child, target_name);
  }
}

pub fn click(browser: &mut Browser, x: f32, y: f32) -> Task<Message> {
  let offset = browser.tabs[browser.active_tab_index].scroll_offset;
  let abs_y = y + offset;

  let mut clicked_href = None;
  let mut clicked_input = None;
  let mut clicked_a_node = None;
  let mut form_submission = None;
  let mut relayout_needed = false;

  {
    let active_tab = &mut browser.tabs[browser.active_tab_index];

    if let Some(doc) = &active_tab.document {
      if let Some(mut current_node) = doc.get_node(x, abs_y) {
        if active_tab.blur() {
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
                  clicked_a_node = Some(current_node.clone());
                }
              } else if e.tag == "input" {
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
        if active_tab.blur() {
          relayout_needed = true;
        }
      }
    }
  }

  if let Some(href) = clicked_href {
    if let Some(node) = clicked_a_node {
      if !browser.tabs[browser.active_tab_index]
        .js_runtime
        .dispatch_event("click", node)
      {
        return Task::none();
      }
    }

    println!("Clicked link: {}", href);
    if href.starts_with('#') {
      return Task::done(Message::NavigateTo(href, None));
    }
    let mut url_handler = URLHandler::default();
    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);
    let resolved = url_handler.resolve(&href);
    return Task::done(Message::NavigateTo(resolved, None));
  } else if let Some(input_node) = clicked_input {
    if !browser.tabs[browser.active_tab_index]
      .js_runtime
      .dispatch_event("click", input_node.clone())
    {
      return Task::none();
    }

    let mut input_type = String::new();
    let mut input_name = None;

    {
      let borrow = input_node.borrow_mut();
      if let Node::Element(elt) = &*borrow {
        input_type = elt
          .attributes
          .get("type")
          .cloned()
          .unwrap_or_else(|| "text".to_string());
        input_name = elt.attributes.get("name").cloned();
      }
    }

    if input_type == "checkbox" || input_type == "radio" {
      if input_type == "radio" {
        if let Some(name) = &input_name {
          let mut scope = None;
          let mut current = Some(Rc::clone(&input_node));

          while let Some(node_rc) = current {
            let is_form = {
              let node = node_rc.borrow();
              if let Node::Element(elt) = &*node {
                elt.tag == "form"
              } else {
                false
              }
            };

            if is_form {
              scope = Some(node_rc.clone());
              break;
            }

            current = {
              let node = node_rc.borrow();
              match &*node {
                Node::Element(elt) => elt.parent.as_ref().and_then(|w| w.upgrade()),
                Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
              }
            }
          }

          if let Some(tree) = &browser.tabs[browser.active_tab_index].tree {
            let search_scope = scope.unwrap_or_else(|| Rc::clone(tree));
            clear_all_radios(&search_scope, name);
          }
        }

        let mut borrow = input_node.borrow_mut();
        if let Node::Element(e) = &mut *borrow {
          e.attributes
            .insert("checked".to_string(), "true".to_string());
        }
      } else if input_type == "checkbox" {
        let mut borrow = input_node.borrow_mut();
        if let Node::Element(e) = &mut *borrow {
          if e.attributes.contains_key("checked") {
            e.attributes.remove("checked");
          } else {
            e.attributes
              .insert("checked".to_string(), "true".to_string());
          }
        }
      }

      browser.relayout();
      return Task::none();
    }

    {
      let mut borrow = input_node.borrow_mut();
      if let Node::Element(e) = &mut *borrow {
        e.attributes
          .insert("data-focused".to_string(), "true".to_string());
        e.attributes
          .insert("data-cursor-visible".to_string(), "true".to_string());
      }
    }
    browser.cursor_blink_visible = true;
    browser.tabs[browser.active_tab_index].focus = Some(input_node);
    browser.relayout();
    return Task::none();
  } else if let Some((action, payload, method)) = form_submission {
    let mut url_handler = URLHandler::default();
    url_handler.init(browser.tabs[browser.active_tab_index].url.clone(), false);
    let resolved = url_handler.resolve(&action);

    if method == "GET" {
      let final_url = if payload.is_empty() {
        resolved
      } else if resolved.contains('?') {
        format!("{}&{}", resolved, payload)
      } else {
        format!("{}?{}", resolved, payload)
      };

      return Task::done(Message::NavigateTo(final_url, None));
    } else {
      return Task::done(Message::NavigateTo(resolved, Some(payload)));
    }
  }

  if relayout_needed {
    browser.relayout();
  }

  Task::none()
}
