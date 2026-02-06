use iced::{Element, Task, window, Subscription};
use iced::widget::{container, canvas};

use crate::app::Message;
use crate::rendering::{DisplayList, lex, layout};
use crate::ui::{BrowserCanvas};

use crate::net::URLHandler;

use std::env;

pub struct Browser {
  pub display_list: DisplayList,
  pub scroll_offset: f32,
  pub current_url: String,
  pub max_y: f32,
  pub raw_body: String,
  pub width: f32,
  pub height: f32,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("file:///home/arshgoyal/Downloads/GOT.txt");
    let args: Vec<String> = env::args().collect();
 
    match args.get(1) {
      Some(value) => {
        url = value.to_string();
      }
      _ => ()
    }
 
    (
      Self {
        display_list: DisplayList::new(),
        scroll_offset: 0.0,
        max_y: 0.0,
        current_url: String::from(url),
        raw_body: String::new(),
        width: 0.0,
        height: 0.0,
      },
      
      Task::done(Message::LoadUrl())
    )
  }
  
  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| {
      Message::WindowResized(size.width, size.height)
    })
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::ScrollChanged(offset) => {
        self.scroll_offset = offset;
        Task::none()
      }
      Message::LoadUrl() => {
        let mut url_handler = URLHandler::default();
        url_handler.init(self.current_url.clone(), false);
        
        let body_result = url_handler.request();
        match body_result {
          Ok(value) => {
            self.raw_body = lex(value, url_handler.view_source);
          }
          _ => ()
        }
        self.display_list = layout(&self.raw_body, self.width);

        self.max_y = self.display_list.items()
          .iter()
          .map(|item| item.y)
          .fold(0.0, f32::max);
        
        Task::none()
      }
      Message::WindowResized(width, height) => {
        self.width = width;
        self.height = height;
        self.display_list = layout(&self.raw_body, self.width);
        
        self.max_y = self.display_list.items()
          .iter()
          .map(|item| item.y)
          .fold(0.0, f32::max);
          
        Task::none()
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let browser_canvas = BrowserCanvas {
      display_list: &self.display_list,
      scroll_offset: self.scroll_offset,
      max_y: self.max_y,
      height: self.height,
    };
    
    let content = canvas(browser_canvas)
      .width(iced::Length::Fill)
      .height(iced::Length::Fill);
    
    container(content)
      .width(iced::Length::Fill)
      .height(iced::Length::Fill)
      .padding(10)
      .into()
  }
}