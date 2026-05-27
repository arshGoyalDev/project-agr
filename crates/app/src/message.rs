use iced::keyboard;

#[derive(Debug, Clone)]
pub enum Message {
  // Window & Canvas Events
  WindowResized(f32, f32),
  TitleBarPressed,
  MinimizeWindow,
  ToggleMaximizeWindow,
  CloseWindow,
  AddressInputChanged(String),
  ScrollChanged(f32),
  Click(f32, f32),
  FontLoaded(Result<(), iced::font::Error>),

  // Navigation
  NavigateTo(String, Option<String>),
  LoadUrl(usize, String, Option<String>, bool, bool),
  HtmlFetched(usize, String, bool, Result<String, String>, bool, bool),
  CssFetched(usize, Vec<String>),
  Reload(usize, String, Option<String>, bool),
  GoBack,
  GoForward,
  ScrollToFragment(String),
  ToggleBookmark,
  ShowResubmitDialog(usize),
  ConfirmResubmit(bool),

  // Tab Management
  NewTab,
  SwitchTab(usize),
  CloseTab(usize, bool),
  TabHovered(usize),
  TabUnhovered,

  // Typing
  KeyPressed(keyboard::Key),
  BlinkCursor,
  TabBlur,
}
