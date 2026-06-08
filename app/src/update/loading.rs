use iced::Task;
use layout::layout::decode_entities;
use net::URLHandler;

use crate::browser::Browser;
use crate::dom::{extract_title, find_inline_styles, find_stylesheet_links};
use crate::message::Message;
use crate::net::{fetch_css_task, fetch_js_task};

use css_parser::CSSParser;
use css_parser::style;
use html_parser::{HTMLParser, ParseYield};
use layout::{DocumentLayout, syntax_highlight};

pub fn html_fetched(
  browser: &mut Browser,
  tab_index: usize,
  _base_url: String, // Kept to avoid unused warnings
  is_view_source: bool,
  result: Result<String, String>,
  _reload: bool,
  _hard_reload: bool,
) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    if let Ok(body) = result {
      if is_view_source {
        // View-Source parses all at once ignoring scripts
        let mut parser = HTMLParser::new(body);
        let tree = loop {
          match parser.resume() {
            ParseYield::Finished(t) => break t,
            _ => continue,
          }
        };

        let highlighted = syntax_highlight(&tree);
        let mut hl_parser = HTMLParser::new(highlighted);
        let final_tree = loop {
          match hl_parser.resume() {
            ParseYield::Finished(t) => break t,
            _ => continue,
          }
        };

        tab.tree = Some(final_tree.clone());
        return Task::done(Message::CssFetched(tab_index, vec![]));
      } else {
        // Normal Load: Start the Resumable Parser!
        let parser = HTMLParser::new(body);
        tab.parser = Some(parser);
        return Task::done(Message::ResumeParsing(tab_index, None));
      }
    } else {
      tab.title = String::from("Network Error");
    }
  }
  Task::none()
}

pub fn resume_parsing(
  browser: &mut Browser,
  tab_index: usize,
  script_body: Option<String>,
) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    // 1. INJECT AND RUN EXTERNAL SCRIPTS
    if let Some(code) = script_body {
      if let Some(parser) = &tab.parser {
        // We must give Boa the partial DOM tree before it can run!
        if let Some(root) = parser.document() {
          tab.js_runtime.set_dom_tree(root);
        }
      }
      tab.js_runtime.run(&code);
    }

    let mut parser = if let Some(p) = tab.parser.take() {
      p
    } else {
      return Task::none();
    };

    loop {
      match parser.resume() {
        ParseYield::Finished(tree) => {
          tab.tree = Some(tree.clone());
          tab.js_runtime.set_dom_tree(tree.clone());

          let mut links = Vec::new();
          find_stylesheet_links(&tree, &mut links);
          let links: Vec<String> = links.into_iter().map(|l| decode_entities(&l)).collect();

          // Fetch any scripts that had the `defer` attribute
          let scripts = parser.deferred_scripts.clone();
          let base_url = tab.url.clone();

          let css_task = if links.is_empty() {
            Task::done(Message::CssFetched(tab_index, vec![]))
          } else {
            Task::perform(
              fetch_css_task(links, base_url.clone(), false, false),
              move |bodies| Message::CssFetched(tab_index, bodies),
            )
          };

          let js_task = if scripts.is_empty() {
            Task::done(Message::JsFetched(tab_index, vec![]))
          } else {
            Task::perform(
              fetch_js_task(scripts, base_url.clone(), false, false),
              move |bodies| Message::JsFetched(tab_index, bodies),
            )
          };

          return Task::batch(vec![css_task, js_task]);
        }
        ParseYield::InlineScript { code } => {
          // 2. INJECT AND RUN INLINE SCRIPTS
          // We must give Boa the partial DOM tree before it can run!
          if let Some(root) = parser.document() {
            tab.js_runtime.set_dom_tree(root);
          }
          tab.js_runtime.run(&code);
        }
        ParseYield::ExternalScript { src } => {
          let url = decode_entities(&src);
          let base_url = tab.url.clone();

          let mut url_handler = URLHandler::default();
          url_handler.init(base_url.clone(), false);
          let resolved_url = url_handler.resolve(&url);

          tab.parser = Some(parser);

          return Task::perform(
            fetch_js_task(vec![resolved_url], base_url, false, false),
            move |mut bodies| {
              let script_code = if !bodies.is_empty() {
                Some(bodies.remove(0))
              } else {
                None
              };
              Message::ResumeParsing(tab_index, script_code)
            },
          );
        }
      }
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

// handles deffered scripts at completion of html parsing
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
