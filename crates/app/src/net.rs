pub async fn fetch_html_task(url: String) -> (String, bool, Result<String, String>) {
  let mut handler = net::URLHandler::default();
  handler.init(url.clone(), false);

  match handler.request() {
    Ok(body) => (url, handler.view_source, Ok(body)),
    Err(_) => (url, handler.view_source, Err("Network Error".to_string())),
  }
}

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
