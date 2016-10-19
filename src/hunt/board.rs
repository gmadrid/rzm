use ansi_term::Colour::{Red,Green};
use result::{Error, Result};
use std::collections::HashSet;
#[cfg(test)]
use std::hash::Hash;
#[cfg(test)]
use std::iter::{FromIterator, IntoIterator};
use super::coord::Coord;
use super::direction::Direction;

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
  dead_cells: HashSet<Coord>,
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

  pub fn alpha() -> Board {
    Board::parse(15, 15,             
      "...............\
       ..X...X.....XX.\
       ..X.....X...X..\
       ........X......\
       ...............\
       ...............\
       .X.X..X.X......\
       .X.X..X........\
       ...X...X...XXX.\
       ...............\
       ...X...........\
       ..X....X...X...\
       X...........X..\
       ..X...X.X....X.\
       ...X...........").unwrap()
  }

  pub fn new(rows: i8, cols: i8, dead_cells: HashSet<Coord>) -> Board {
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
      dead_cells: dead_cells,
    }
  }

  pub fn dump(&self) {
    for row in 0..self.num_rows {
      print!("{:2} ", row);
      for col in 0..self.num_cols {
        let c = Coord::new(row, col);
        if self.possible_cells.contains(&c) {
          print!("{}", Green.paint("O"));
        } else if self.dead_cells.contains(&c) {
          print!("{}", Red.paint("x"));
        } else {
          print!("{}", ".");
        }
      }
      println!("");
    }
  }

  #[cfg(test)]
  fn rows(&self) -> i8 {
    self.num_rows
  }

  #[cfg(test)]
  fn cols(&self) -> i8 {
    self.num_cols
  }

  #[cfg(test)]
  fn possible_locations(&self) -> HashSet<Coord> {
    self.possible_cells.clone()
  }

  #[cfg(test)]
  fn coords_for_string(&self, s: &str, ch: char) -> Result<Vec<Coord>> {
    let mut vec = Vec::with_capacity((self.num_rows * self.num_cols) as usize);
    let mut i = s.chars();
    for row in 0..self.num_rows {
      for col in 0..self.num_cols {
        let sch = try!(i.next().ok_or(Error::BoardParseError));
        if ch == sch {
          vec.push(Coord::new(row, col))
        }
      }
    }
    Ok(vec)
  }

  fn is_valid_coord(&self, coord: Coord) -> bool {
    coord.row >= 0 && coord.row < self.num_rows && coord.col >= 0 && coord.col < self.num_cols &&
    !self.dead_cells.contains(&coord)
  }

  pub fn north(&mut self) {
    self.move_in_direction(Direction::North);
  }

  pub fn south(&mut self) {
    self.move_in_direction(Direction::South);
  }

  pub fn east(&mut self) {
    self.move_in_direction(Direction::East);
  }

  pub fn west(&mut self) {
    self.move_in_direction(Direction::West);
  }

  fn move_in_direction(&mut self, dir: Direction) {
    let mut new_active_cells = HashSet::<Coord>::new();
    for coord in self.possible_cells.iter() {
      let new_coord = coord.move_in_direction(dir);
      if self.is_valid_coord(new_coord) {
        new_active_cells.insert(new_coord);
      }
    }
    self.possible_cells = new_active_cells;
  }
}

// --------- Test code ---------

macro_rules! assert_has_coords {
    ( $coords:expr , $($c:expr), *) => {
      {
        let coord_set_in = HashSet::<Coord>::from_iter($coords.into_iter());

        let mut coord_set_to_check = HashSet::<Coord>::new();
        $(
          let (r, c) = $c;
          let coord = Coord::new(r, c);
          coord_set_to_check.insert(coord);
          )*
        // TODO: better error reporting when lengths don't match.
        assert_eq!(coord_set_in.len(), coord_set_to_check.len());
        // TODO: better error reporting when the sets aren't equal.
        assert!(coord_set_in == coord_set_to_check);
      }
    }
}

#[cfg(test)]
fn assert_eq_sets<T, S, I>(s1: T, s2: S)
  where I: Eq + Hash,
        T: IntoIterator<Item = I>,
        S: IntoIterator<Item = I> {
  let set1: HashSet<T::Item> = HashSet::from_iter(s1.into_iter());
  let set2: HashSet<S::Item> = HashSet::from_iter(s2.into_iter());
  assert_eq!(set1.len(), set2.len());
  assert!(set1 == set2);
}

#[test]
fn test_board_size() {
  let b1 = board!(3; 4);
  assert_eq!(3, b1.rows());
  assert_eq!(4, b1.cols());
  assert_eq!(12, b1.possible_locations().len());

  let b2 = board!(7; 9);
  assert_eq!(7, b2.rows());
  assert_eq!(9, b2.cols());
  assert_eq!(63, b2.possible_locations().len());
}

#[test]
fn test_board_macro() {
  let board = board!(2; 3);
  assert_eq!(2, board.rows());
  assert_eq!(3, board.cols());
  assert_has_coords!(board.possible_locations(),
                     (0, 0),
                     (0, 1),
                     (0, 2),
                     (1, 0),
                     (1, 1),
                     (1, 2));
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("......", '.').unwrap());
}

#[test]
fn test_board_with_islands() {
  let board = board!(3; 4; (1, 2), (2, 3), (0, 0));
  assert_eq!(3, board.rows());
  assert_eq!(4, board.cols());
  assert_eq!(9, board.possible_locations().len());
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("X.....X....X", '.').unwrap());
}

#[test]
fn test_parse() {
  let board = Board::parse(2, 3, "......").unwrap();
  assert_eq!(6, board.possible_locations().len());
  assert_has_coords!(board.possible_locations(),
                     (0, 0),
                     (0, 1),
                     (0, 2),
                     (1, 0),
                     (1, 1),
                     (1, 2));

  let board = Board::parse(2, 3, "..X..X").unwrap();
  assert_eq!(4, board.possible_locations().len());
  assert_has_coords!(board.possible_locations(), (0, 0), (0, 1), (1, 0), (1, 1));

  // TODO: check too short string
  // TODO: check too long string
}

#[test]
fn test_string_to_coords() {
  let str = "..XX..";
  let board = Board::parse(2, 3, str).unwrap();
  let dot_coords = board.coords_for_string(str, '.').unwrap();
  assert_eq!(4, dot_coords.len());
  assert_has_coords!(dot_coords, (0, 0), (0, 1), (1, 1), (1, 2));

  let x_coords = board.coords_for_string(str, 'X').unwrap();
  assert_eq!(2, x_coords.len());
  assert_has_coords!(x_coords, (0, 2), (1, 0));
}

#[test]
fn test_is_valid_coord() {
  let board = Board::parse(2, 3, "..XX..").unwrap();
  assert!(board.is_valid_coord(Coord::new(0, 0)));
  assert!(board.is_valid_coord(Coord::new(1, 2)));
  assert!(!board.is_valid_coord(Coord::new(0, 2))); // island
  assert!(!board.is_valid_coord(Coord::new(1, 0))); // island
  assert!(!board.is_valid_coord(Coord::new(-1, 1)));
  assert!(!board.is_valid_coord(Coord::new(2, -1)));
  assert!(!board.is_valid_coord(Coord::new(3, 1)));
  assert!(!board.is_valid_coord(Coord::new(2, 2)));
}

#[test]
fn test_move() {
  let mut board = Board::parse(2, 3, "..XX..").unwrap();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("..XX..", '.').unwrap());

  board.north();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("X.XXXX", '.').unwrap());
  board.west();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string(".XXXXX", '.').unwrap());

  let mut board = Board::parse(2, 3, "..XX..").unwrap();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("..XX..", '.').unwrap());
  board.south();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("XXXX.X", '.').unwrap());
  board.east();
  assert_eq_sets(board.possible_locations(),
                 board.coords_for_string("XXXXX.", '.').unwrap());

}
