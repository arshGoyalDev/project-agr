#[derive(Debug, Clone)]
pub enum Message {
  ScrollChanged(f32),
  WindowResized(f32, f32),
  Click(f32, f32),
  NavigateTo(String),
  LoadUrl(String),
  AddressInputChanged(String),
  NewTab,
  SwitchTab(usize),
  GoBack,
  GoForward,
  TitleBarPressed,
  MinimizeWindow,
  ToggleMaximizeWindow,
  CloseWindow,
}
