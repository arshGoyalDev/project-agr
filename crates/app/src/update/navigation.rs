use crate::browser::Browser;
use crate::message::Message;
use crate::net::{fetch_html_task};
use iced::Task;

pub fn navigate_to(browser: &mut Browser, url: String) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  tab.history.truncate(tab.history_index + 1);

  if tab.history.last() != Some(&url) {
    tab.history.push(url.clone());
    tab.history_index = tab.history.len() - 1;
  }

  Task::done(Message::LoadUrl(browser.active_tab_index, url))
}

pub fn go_back(browser: &mut Browser) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  if tab.history_index > 0 {
    tab.history_index -= 1;
    let prev_url = tab.history[tab.history_index].clone();
    return Task::done(Message::LoadUrl(browser.active_tab_index, prev_url));
  }

  Task::none()
}

pub fn go_forward(browser: &mut Browser) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  if tab.history_index + 1 < tab.history.len() {
    tab.history_index += 1;
    let next_url = tab.history[tab.history_index].clone();
    return Task::done(Message::LoadUrl(browser.active_tab_index, next_url));
  }

  Task::none()
}

pub fn load_url(browser: &mut Browser, tab_index: usize, url: String) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    tab.url = url.clone();
    if tab_index == browser.active_tab_index {
      browser.address_bar_text = url.clone();
    }
    tab.title = String::from("Loading...");
    tab.display_list = layout::DisplayList::new();
  }

  Task::perform(fetch_html_task(url), move |(base_url, is_view_source, result)| {
    Message::HtmlFetched(tab_index, base_url, is_view_source, result)
  })
}