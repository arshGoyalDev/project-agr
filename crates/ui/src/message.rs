#[derive(Debug, Clone)]
pub enum Message {
  ScrollChanged(f32),
  LoadUrl(),
  WindowResized(f32, f32),
  Click(f32, f32),
}
