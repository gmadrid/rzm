#[derive(Clone,Copy,Hash)]
pub enum Direction {
	North,
	South,
	East,
	West
}

pub fn xdelta(dir: Direction) -> i8 {
	match dir {
		Direction::East => 1,
		Direction::West => -1,
		Direction::North => 0,
		Direction::South => 0
	}
}

pub fn ydelta(dir: Direction) -> i8 {
	match dir {
		Direction::East => 0,
		Direction::West => 0,
		Direction::North => -1,
		Direction::South => 1
	}
}

#[test]
fn testDirection() {
	assert_eq!(0, xdelta(Direction::North));
	assert_eq!(0, xdelta(Direction::South));
	assert_eq!(1, xdelta(Direction::East));
	assert_eq!(-1, xdelta(Direction::West));
	assert_eq!(-1, ydelta(Direction::North));
	assert_eq!(1, ydelta(Direction::South));
	assert_eq!(0, ydelta(Direction::East));
	assert_eq!(0, ydelta(Direction::West));
}