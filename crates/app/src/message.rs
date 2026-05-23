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

  // Navigation
  NavigateTo(String),
  LoadUrl(usize, String, Option<String>, bool, bool),
  HtmlFetched(usize, String, bool, Result<String, String>, bool, bool),
  CssFetched(usize, Vec<String>),
  Reload(usize, String, Option<String>, bool),
  GoBack,
  GoForward,
  ScrollToFragment(String),
  ToggleBookmark,

  // Tab Management
  NewTab,
  SwitchTab(usize),
  CloseTab(usize, bool),
  TabHovered(usize),
  TabUnhovered,

  // Typing
  KeyPressed(char),
  BackspacePressed,
  BlinkCursor,
  EnterPressed,
}
