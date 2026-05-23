#[derive(Debug, Clone)]
pub enum Message {
  // window
  WindowResized(f32, f32),
  TitleBarPressed,
  MinimizeWindow,
  ToggleMaximizeWindow,
  CloseWindow,

  // web
  NavigateTo(String),
  LoadUrl(usize, String, Option<String>, bool, bool),
  HtmlFetched(usize, String, bool, Result<String, String>, bool, bool),
  CssFetched(usize, Vec<String>),
  Reload(usize, String, Option<String>, bool),

  // tab
  NewTab,
  SwitchTab(usize),
  GoBack,
  GoForward,
  AddressInputChanged(String),
  CloseTab(usize, bool),
  TabHovered(usize),
  TabUnhovered,
  ToggleBookmark,
  ScrollChanged(f32),
  ScrollToFragment(String),

  // inputs
  Click(f32, f32),
  KeyPressed(char),
  BackspacePressed,
  BlinkCursor,
}
