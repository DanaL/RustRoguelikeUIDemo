extern crate rand;

use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
	Blank,
	Wall,
	Tree,
	Dirt,
	Grass,
	Player,
	Water,
}

pub fn make_rando_test_dungeon() -> Vec<Vec<Tile>> {
	let mut dungeon = vec![vec![Tile::Wall; 30]];

	for _ in 0..28 {
		let mut row = vec![Tile::Wall];
		for _ in 0..28 {
			let x = rand::thread_rng().gen_range(0, 3);
			if x == 0 {
				row.push(Tile::Tree);
			} else if x == 1 {
				row.push(Tile::Grass);
			} else {
				row.push(Tile::Dirt);
			}
		}
		row.push(Tile::Wall);
		dungeon.push(row);
	}

	for _ in 0..3 {
		let r = rand::thread_rng().gen_range(5, 24);
		let cc = rand::thread_rng().gen_range(5, 24);

		for rd in -1..2 {
			let cs = cc - rand::thread_rng().gen_range(2, 5);
			let ce = cc + rand::thread_rng().gen_range(2, 5);
			for c in cs..ce+1 {
				dungeon[(r + rd as i32) as usize][c] = Tile::Water;
			}
		}	
	}

	for _ in 0..3 {
		let r = rand::thread_rng().gen_range(5, 25);
		let c = rand::thread_rng().gen_range(5, 25);
		let n = rand::thread_rng().gen_range(0, 2);
		if n == 0 {
			for c2 in c..c+4 {
				dungeon[r][c2] = Tile::Wall;
			}
		} else {
			for r2 in r..r+4 {
				dungeon[r2][c] = Tile::Wall;
			}
		}
	}

	dungeon[1][4] = Tile::Wall;
	dungeon[2][4] = Tile::Wall;
	dungeon[3][4] = Tile::Wall;
	dungeon[4][4] = Tile::Wall;

	dungeon.push(vec![Tile::Wall; 30]);

	dungeon
}

