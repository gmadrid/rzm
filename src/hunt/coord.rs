use self::super::direction::{Direction, xdelta, ydelta};
use std::collections::HashSet;

#[derive(Eq,PartialEq,Hash,Copy,Clone,Debug)]
pub struct Coord {
  row: i8,
  col: i8,
}

impl Coord {
  pub fn new(row: i8, col: i8) -> Coord {
    Coord {
      row: row,
      col: col,
    }
  }

  pub fn in_direction(&self, dir: Direction) -> Coord {
    Coord::new(self.row + xdelta(dir), self.col + ydelta(dir))
  }
}

#[test]
fn test_coord_equals() {
  let c1 = Coord::new(1, 1);
  let c2 = Coord::new(1, 1);
  let c3 = Coord::new(1, 2);
  let c4 = Coord::new(2, 1);

  assert_eq!(c1, c2);
  assert!(c1 != c3);
  assert!(c2 != c4);
  assert!(c3 != c4);
}

#[test]
fn test_coord_direction() {
  let c = Coord::new(5, 5);

  assert_eq!(Coord::new(4, 5), c.in_direction(Direction::West));
  assert_eq!(Coord::new(6, 5), c.in_direction(Direction::East));
  assert_eq!(Coord::new(5, 4), c.in_direction(Direction::North));
  assert_eq!(Coord::new(5, 6), c.in_direction(Direction::South));
}

#[test]
fn test_hash() {
  let c1 = Coord::new(1, 1);
  let c2 = Coord::new(1, 1);
  let c3 = Coord::new(1, 2);

  let mut set = HashSet::<Coord>::new();
  set.insert(c1);
  assert!(set.contains(&c1));
  assert!(set.contains(&c2));
  assert!(!set.contains(&c3));
}
