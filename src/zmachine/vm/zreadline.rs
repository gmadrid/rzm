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
  startx: i32,
  starty: i32,

  max_length: usize,
  cursor_pos: i32,
}

impl ZReadline {
  pub fn new(window: WINDOW, max_length: usize) -> ZReadline {
    let mut startx = 0i32;
    let mut starty = 0i32;
    getyx(window, &mut starty, &mut startx);

    ZReadline {
      window: window,
      startx: startx,
      starty: starty,
      max_length: max_length,
      cursor_pos: 0,
    }
  }

  pub fn readline(self) -> String {
    let mut input = String::new();
    loop {
      let ch = wgetch(self.window);
      input.push(ch as u8 as char);
      waddch(self.window, ch as chtype);
      match ch as u8 as char {
        '\n' => return input,
        _ => {}
      }
    }
  }
}
