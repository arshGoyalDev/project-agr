use crate::browser::Browser;
use crate::message::Message;
use crate::tab::Tab;
use iced::Task;

pub fn new_tab(browser: &mut Browser) -> Task<Message> {
  browser.tabs.push(Tab::new("about:blank".to_string()));
  browser.active_tab_index = browser.tabs.len() - 1;
  browser.address_bar_text = "about:blank".to_string();
  Task::done(Message::LoadUrl(
    browser.active_tab_index,
    "about:blank".to_string(),
  ))
}

pub fn switch_tab(browser: &mut Browser, index: usize) -> Task<Message> {
  if index < browser.tabs.len() {
    browser.active_tab_index = index;
    browser.address_bar_text = browser.tabs[index].url.clone();
  }
  Task::none()
}

pub fn close_tab(browser: &mut Browser, index: usize) -> Task<Message> {
  if browser.tabs.len() > 1 {
    browser.tabs.remove(index);

    if browser.active_tab_index >= index && browser.active_tab_index > 0 {
      browser.active_tab_index -= 1;
    } else if browser.active_tab_index >= browser.tabs.len() {
      browser.active_tab_index = browser.tabs.len() - 1;
    }

    browser.address_bar_text = browser.tabs[browser.active_tab_index].url.clone();
    Task::none()
  } else {
    // Last tab — replace with blank rather than exiting
    browser.tabs[0] = Tab::new("about:blank".to_string());
    browser.address_bar_text = "about:blank".to_string();
    Task::done(Message::LoadUrl(0, "about:blank".to_string()))
  }
}

pub fn tab_hovered(browser: &mut Browser, index: usize) -> Task<Message> {
  browser.hovered_tab = Some(index);
  Task::none()
}

pub fn tab_unhovered(browser: &mut Browser) -> Task<Message> {
  browser.hovered_tab = None;
  Task::none()
}