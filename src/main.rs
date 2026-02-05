mod url_handler;
use url_handler::{URLHandler, load};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut url = String::from("file:///home/username/Downloads/some.txt");
    
    match args.get(1) {
        Some(value) => {
            url = value.to_string();
        }
        _ => ()
    }
    
    let mut handler = URLHandler::default();
    handler.init(url, false);
    
    match load(handler) {
        Ok(_) => {},
        Err(e) => eprintln!("Error: {}", e),
    }
}