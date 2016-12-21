use ncurses::*;

// For now, assume ncurses.
//
// Remember starting cursor position.
// Get passed max string length.
//
// On each character:
//   update string,
//   update display.

pub struct ZReadline {
  window: WINDOW,
}

const NL: i32 = '\n' as i32;
const BS: i32 = 0x7fi32;

impl ZReadline {
  pub fn new(window: WINDOW) -> ZReadline {
    let mut startx = 0i32;
    let mut starty = 0i32;
    getyx(window, &mut starty, &mut startx);

    ZReadline { window: window }
  }

  pub fn readline(self) -> String {
    let mut input = String::new();
    loop {
      let ch = wgetch(self.window);
      match ch {
        NL => {
          input.push('\n');
          waddch(self.window, ch as chtype);
          return input;
        }
        BS => {
          if input.len() > 0 {
            let mut x = 0i32;
            let mut y = 0i32;
            getyx(self.window, &mut y, &mut x);
            input.pop();
            mvwdelch(self.window, y, x - 1);
          }
        }
        _ => {
          input.push(ch as u8 as char);
          waddch(self.window, ch as chtype);
        }
      }
    }
  }
}
