use crate::rendering::DisplayList;

const HSTEP: f32 = 9.0;
const VSTEP: f32 = 15.0;

pub fn layout(text: &str, width: f32) -> DisplayList {
  let mut display_list = DisplayList::new();
  let mut cursor_x = HSTEP;
  let mut cursor_y = VSTEP;

  for c in text.chars() {
    if c == '\n' {
      cursor_y += VSTEP * 1.5;
      cursor_x = HSTEP;
    } else {
      display_list.add_item(cursor_x, cursor_y, c);
      cursor_x += HSTEP;

      if cursor_x >= width - HSTEP {
        cursor_y += VSTEP;
        cursor_x = HSTEP;
      }
    }
  }

  display_list
}