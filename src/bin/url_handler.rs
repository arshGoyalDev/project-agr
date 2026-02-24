use project_agr::net::url_handler;
use std::io::{self, Write};

fn main() {
  let mut url_h = url_handler::URLHandler::default();
  
  loop {
    print!("[Client]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let url = input.trim();
    
    if url == "q" {
      break;
    }
    
    url_h.init(url.to_string(), false);
    
    match url_handler::load(&mut url_h) {
      Ok(()) => {
        println!("\nSuccessfully loaded URL\n");
      }
      Err(e) => {
        println!("Error loading URL: {}\n", e);
      }
    }
  }
}