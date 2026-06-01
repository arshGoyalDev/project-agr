use iced::Task;
use layout::layout::decode_entities;

use crate::browser::Browser;
use crate::dom::{extract_title, find_inline_styles, find_script_links, find_stylesheet_links};
use crate::message::Message;
use crate::net::{fetch_css_task, fetch_js_task};

use css_parser::CSSParser;
use css_parser::style;
use html_parser::HTMLParser;
use layout::{DocumentLayout, syntax_highlight};

pub fn html_fetched(
  browser: &mut Browser,
  tab_index: usize,
  base_url: String,
  is_view_source: bool,
  result: Result<String, String>,
  reload: bool,
  hard_reload: bool,
) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    if let Ok(body) = result {
      let mut html_parser = HTMLParser::new(body);
      let mut tree = html_parser.parse();

      if is_view_source {
        let highlighted = syntax_highlight(&tree);
        tree = HTMLParser::new(highlighted).parse();
      }

      let mut links = Vec::new();
      find_stylesheet_links(&tree, &mut links);
      let links: Vec<String> = links.into_iter().map(|l| decode_entities(&l)).collect();

      let mut scripts = Vec::new();
      find_script_links(&tree, &mut scripts);
      let scripts: Vec<String> = scripts.into_iter().map(|s| decode_entities(&s)).collect();

      tab.tree = Some(tree.clone());
      tab.js_runtime.set_dom_tree(tree.clone());

      // Create the CSS Fetching Task
      let css_task = if links.is_empty() {
        Task::done(Message::CssFetched(tab_index, vec![]))
      } else {
        Task::perform(
          fetch_css_task(links, base_url.clone(), reload, hard_reload),
          move |bodies| Message::CssFetched(tab_index, bodies),
        )
      };

      // Create the JS Fetching Task
      let js_task = if scripts.is_empty() {
        Task::done(Message::JsFetched(tab_index, vec![]))
      } else {
        Task::perform(
          fetch_js_task(scripts, base_url.clone(), reload, hard_reload),
          move |bodies| Message::JsFetched(tab_index, bodies),
        )
      };

      // Run both tasks concurrently and return them
      return Task::batch(vec![css_task, js_task]);
    } else {
      tab.title = String::from("Network Error");
    }
  }

  Task::none()
}
pub fn css_fetched(
  browser: &mut Browser,
  tab_index: usize,
  css_bodies: Vec<String>,
) -> Task<Message> {
  let width = browser.width;

  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    if let Some(tree) = &tab.tree {
      let default_css = include_str!("../../../assets/browser.css").to_string();
      let mut css_parser = CSSParser::new(&default_css);
      let mut rules = css_parser.parse();

      for body in css_bodies {
        let mut linked_parser = CSSParser::new(&body);
        rules.extend(linked_parser.parse());
      }

      let mut inline_style_texts = Vec::new();
      find_inline_styles(tree, &mut inline_style_texts);
      for css_text in inline_style_texts {
        rules.extend(CSSParser::new(&css_text).parse());
      }

      rules.sort_by_key(|r| r.priority);
      style(tree, &rules);

      tab.title = extract_title(tree).unwrap_or_else(|| tab.url.clone());

      let mut doc = DocumentLayout::new(tree);
      doc.layout(width);

      tab.display_list = doc.paint();
      tab.max_y = tab.display_list.max_y();
      tab.document = Some(doc);

      if let Some(fragment) = tab.url.split('#').nth(1) {
        if let Some(y_pos) = tab.document.as_ref().unwrap().get_element_y(fragment) {
          let max_scroll = (tab.max_y - width).max(0.0);
          tab.scroll_offset = y_pos.clamp(0.0, max_scroll);
        } else {
          tab.scroll_offset = 0.0;
        }
      } else {
        tab.scroll_offset = 0.0;
      }
    }
  }

  Task::none()
}

pub fn js_fetched(
  browser: &mut Browser,
  tab_index: usize,
  js_bodies: Vec<String>,
) -> Task<Message> {
  let mut needs_relayout = false;

  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    for body in js_bodies {
      if tab.js_runtime.run(&body) {
        needs_relayout = true;
      }
    }

    if needs_relayout {
      if let Some(tree) = &tab.tree {
        let width = browser.width;
        let mut doc = DocumentLayout::new(&tree.clone());
        doc.layout(width);
        tab.display_list = doc.paint();
        tab.max_y = tab.display_list.max_y();
        tab.document = Some(doc);
      }
    }
  }

  Task::none()
}
