use self::super::direction::{Direction, xdelta, ydelta};

#[derive(Eq,PartialEq,Hash,Copy,Clone,Debug)]
pub struct Coord {
	row: i8,
	col: i8
}

impl Coord {
	fn new(row: i8, col: i8) -> Coord {
		Coord { row: row, col: col }
	}

	fn in_direction(&self, dir: Direction) -> Coord {
		Coord::new(self.row + xdelta(dir), self.col + ydelta(dir))
	}
}

#[test]
fn testCoordEquals() {
	let c1 = Coord::new(1, 1);
	let c2 = Coord::new(1, 1);
	let c3 = Coord::new(1, 2);
	let c4 = Coord::new(2, 1);

	assert_eq!(c1, c2);
	assert!(c1 != c3);
	assert!(c2 != c4);
	assert!(c3 != c4);
}