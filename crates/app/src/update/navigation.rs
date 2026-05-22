use crate::browser::Browser;
use crate::message::Message;
use crate::net::fetch_html_task;
use iced::Task;

pub fn navigate_to(browser: &mut Browser, url: String) -> Task<Message> {
  let active_tab = &browser.tabs[browser.active_tab_index];
  let current_url = active_tab.url.clone();

  let final_url = if url.contains("://") || url.starts_with("about:") || url.starts_with("data:") {
    url.clone()
  } else if url.starts_with('#') {
    let base = current_url.split('#').next().unwrap_or(&current_url);
    format!("{}{}", base, url)
  } else if url.contains('.') && !url.contains(' ') {
    format!("https://{}", url)
  } else {
    let query = url.replace(' ', "+");
    format!("https://google.com/search?q={}", query)
  };

  let current_base = current_url.split('#').next().unwrap_or(&current_url);
  let new_base = final_url.split('#').next().unwrap_or(&final_url);

  if current_base == new_base && final_url.contains('#') && active_tab.document.is_some() {
    let fragment = final_url.split('#').nth(1).unwrap().to_string();

    let tab = &mut browser.tabs[browser.active_tab_index];
    tab.url = final_url.clone();
    browser.address_bar_text = final_url.clone();

    tab.history.truncate(tab.history_index + 1);
    if tab.history.last() != Some(&final_url) {
      tab.history.push(final_url);
      tab.history_index = tab.history.len() - 1;
    }

    return Task::done(Message::ScrollToFragment(fragment));
  }

  let tab = &mut browser.tabs[browser.active_tab_index];
  tab.history.truncate(tab.history_index + 1);

  if tab.history.last() != Some(&final_url) {
    tab.history.push(final_url.clone());
    tab.history_index = tab.history.len() - 1;
  }

  Task::done(Message::LoadUrl(browser.active_tab_index, final_url, None))
}

pub fn scroll_to_fragment(browser: &mut Browser, fragment: String) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  if let Some(doc) = &tab.document {
    if let Some(y_pos) = doc.get_element_y(&fragment) {
      let max_scroll = (tab.max_y - browser.height).max(0.0);
      tab.scroll_offset = y_pos.clamp(0.0, max_scroll);
    }
  }
  Task::none()
}

pub fn go_back(browser: &mut Browser) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  if tab.history_index > 0 {
    tab.history_index -= 1;
    let prev_url = tab.history[tab.history_index].clone();
    return Task::done(Message::LoadUrl(browser.active_tab_index, prev_url, None));
  }

  Task::none()
}

pub fn go_forward(browser: &mut Browser) -> Task<Message> {
  let tab = &mut browser.tabs[browser.active_tab_index];

  if tab.history_index + 1 < tab.history.len() {
    tab.history_index += 1;
    let next_url = tab.history[tab.history_index].clone();
    return Task::done(Message::LoadUrl(browser.active_tab_index, next_url, None));
  }

  Task::none()
}

pub fn toggle_bookmark(browser: &mut Browser) -> Task<Message> {
  let url = browser.tabs[browser.active_tab_index].url.clone();

  if !url.is_empty() && url != "about:blank" {
    if let Some(pos) = browser.bookmarks.iter().position(|x| x == &url) {
      browser.bookmarks.remove(pos);
    } else {
      browser.bookmarks.push(url);
    }
  }
  Task::none()
}

// Pass the payload dynamically to fetch_html_task
pub fn load_url(
  browser: &mut Browser,
  tab_index: usize,
  url: String,
  payload: Option<String>,
) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    tab.url = url.clone();
    if tab_index == browser.active_tab_index {
      browser.address_bar_text = url.clone();
    }
    tab.title = String::from("Loading...");
    tab.display_list = layout::DisplayList::new();
  }

  if url == "about:bookmarks" {
    let mut html = String::from(
      "<html><head><title>Bookmarks</title></head><body style=\"padding: 20px;\"><h1>My Bookmarks</h1><ul>",
    );

    if browser.bookmarks.is_empty() {
      html.push_str("<li>No bookmarks saved yet!</li>");
    } else {
      for b in &browser.bookmarks {
        html.push_str(&format!("<li><a href=\"{}\">{}</a></li>", b, b));
      }
    }
    html.push_str("</ul></body></html>");

    return Task::done(Message::HtmlFetched(
      tab_index,
      url.clone(),
      false,
      Ok(html),
    ));
  }

  Task::perform(
    fetch_html_task(url, payload), // <--- Updates here
    move |(base_url, is_view_source, result)| {
      Message::HtmlFetched(tab_index, base_url, is_view_source, result)
    },
  )
}
