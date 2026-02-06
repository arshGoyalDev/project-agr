#[derive(Debug, Clone)]
pub enum Message {
  ScrollChanged(f32),
  LoadUrl(),
  WindowResized(f32, f32)
}