pub async fn fetch_html_task(
  url: String,
  payload: Option<String>,
  reload: bool,
  hard_reload: bool,
) -> (String, bool, Result<String, String>) {
  let mut url_handler = net::URLHandler::default();
  url_handler.init(url.clone(), false);

  match url_handler.request(payload.as_deref(), reload, hard_reload) {
    Ok(body) => (url, url_handler.view_source, Ok(body)),
    Err(_) => (
      url,
      url_handler.view_source,
      Err("Network Error".to_string()),
    ),
  }
}

pub async fn fetch_css_task(
  links: Vec<String>,
  base_url: String,
  reload: bool,
  hard_reload: bool,
) -> Vec<String> {
  let mut css_bodies = Vec::new();

  for link in links {
    let mut url_handler = net::URLHandler::default();
    url_handler.init(base_url.clone(), false);
    let resolved_url = url_handler.resolve(&link);

    let mut url_handler = net::URLHandler::default();
    url_handler.init(resolved_url, false);

    if let Ok(css_body) = url_handler.request(None, reload, hard_reload) {
      css_bodies.push(css_body);
    }
  }

  css_bodies
}

pub async fn fetch_js_task(
  scripts: Vec<String>,
  base_url: String,
  reload: bool,
  hard_reload: bool,
) -> Vec<String> {
  let mut js_bodies = Vec::new();

  for script in scripts {
    let mut url_handler = net::URLHandler::default();
    url_handler.init(base_url.clone(), false);
    let resolved_url = url_handler.resolve(&script);

    let mut url_handler = net::URLHandler::default();
    url_handler.init(resolved_url, false);

    if let Ok(js_body) = url_handler.request(None, reload, hard_reload) {
      js_bodies.push(js_body);
    }
  }

  js_bodies
}
