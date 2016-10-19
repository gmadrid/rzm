use result::{Error, Result};
use std::collections::HashSet;
use super::coord::Coord;

macro_rules! board {
	( $rows:expr ; $cols:expr) => { Board::new($rows, $cols, HashSet::<Coord>::new()) };
    ( $rows:expr ; $cols:expr ; $($c:expr), *) => {
    	{
    		let mut dead_cells = HashSet::<Coord>::new();
    		$(
    			let (r, c) = $c;
    			let coord = Coord::new(r, c);
    			dead_cells.insert(coord);
    			)*
    		let b = Board::new($rows, $cols, dead_cells);
    		b
	    }
    }
}

pub struct Board {
  num_rows: i8,
  num_cols: i8,
  possible_cells: HashSet<Coord>,
}

impl Board {
  fn parse(rows: i8, cols: i8, s: &str) -> Result<Board> {
    let mut dead_cells = HashSet::<Coord>::new();
    let mut i = s.chars();
    for row in 0..rows {
      for col in 0..cols {
        // TODO: make this a cloruse, or you create the error on every iteration.
        let ch = try!(i.next().ok_or(Error::BoardParseError));
        if ch == 'X' {
          dead_cells.insert(Coord::new(row, col));
        }
      }
    }
    // TODO: check for no more chars here
    Ok(Board::new(rows, cols, dead_cells))
  }

  fn new(rows: i8, cols: i8, dead_cells: HashSet<Coord>) -> Board {
    let mut possible_cells = HashSet::<Coord>::new();
    for row in 0..rows {
      for col in 0..cols {
        let c = Coord::new(row, col);
        if !dead_cells.contains(&c) {
          possible_cells.insert(c);
        }
      }
    }
    Board {
      num_rows: rows,
      num_cols: cols,
      possible_cells: possible_cells,
    }
  }

  fn rows(&self) -> i8 {
    self.num_rows
  }

  fn cols(&self) -> i8 {
    self.num_cols
  }

  fn possible_locations(&self) -> HashSet<Coord> {
    self.possible_cells.clone()
  }
}

#[test]
fn test_board_size() {
  let b1 = board!(3; 4);
  assert_eq!(3, b1.rows());
  assert_eq!(4, b1.cols());

  let b2 = board!(7; 9);
  assert_eq!(7, b2.rows());
  assert_eq!(9, b2.cols());
}

#[test]
fn test_board_macro() {
  let board = board!(2; 3);
  assert_eq!(2, board.rows());
  assert_eq!(3, board.cols());
}

#[test]
fn test_empty_board() {
  let board = board!(3; 4);
  assert_eq!(3, board.rows());
  assert_eq!(4, board.cols());
  assert_eq!(12, board.possible_locations().len());
  // TODO: check the contents of possible_locations.
}

#[test]
fn test_board_with_islands() {
  let board = board!(3; 4; (1, 2), (2, 3), (0, 0));
  assert_eq!(3, board.rows());
  assert_eq!(4, board.cols());
  assert_eq!(9, board.possible_locations().len());
  // TODO: check the contents of possible_locations.
}

#[test]
fn test_parse() {
  let board = Board::parse(2, 3, "......").unwrap();
  assert_eq!(6, board.possible_locations().len());

  let board = Board::parse(2, 3, "..X..X").unwrap();
  assert_eq!(4, board.possible_locations().len());

  // TODO: check too short string
  // TODO: check too long string
}
