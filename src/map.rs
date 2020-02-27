extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use std::f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
	Blank,
	Wall,
	Tree,
	Dirt,
	Grass,
	Player,
	Water,
	DeepWater,
	Sand,
	Mountain,
	SnowPeak,
	Gate,
	StoneFloor,
}

pub fn in_bounds(map: &Vec<Vec<Tile>>, r: i32, c: i32) -> bool {
	let width = map.len() as i32;
	r >= 0 && c >= 0 && r < width && c < width
}

pub fn is_clear(tile: Tile) -> bool {
	match tile {
		Tile::Wall | Tile::Blank | Tile::Mountain | Tile::SnowPeak => false,
		_ => true,
	}
}

pub fn is_passable(tile: Tile) -> bool {
	match tile {
		Tile::DeepWater | Tile::Wall | Tile::Blank |
		Tile::Mountain | Tile::SnowPeak | Tile::Gate => false,
		_ => true,
	}
}

fn val_to_terrain(val: f32) -> Tile {
	if val < -0.5 {
		return Tile::DeepWater;
	} else if val < -0.25 {
		return Tile::Water;
	} else if val < 0.20 {
		return Tile::Sand;	
	} else if val < 0.45 {
		return Tile::Grass;
	} else if val < 0.85 {
		return Tile::Tree;
	} else if val < 1.5 {
		return Tile::Mountain;
	}

	Tile::SnowPeak
}

fn fuzz(width: usize, scale: f32) -> f32 {
	(rand::thread_rng().gen_range(0.0, 1.0) * 2f32 - 1f32) * width as f32 * scale	
}

fn diamond_step(grid: &mut Vec<Vec<f32>>, r: usize, c: usize, width: usize, scale: f32) {
	let mut avg = grid[r][c];
	avg += grid[r][c + width - 1];
	avg += grid[r + width - 1][c];
	avg += grid[r + width - 1][c + width - 1];
	avg /= 4f32;

	grid[r + width /2][c + width / 2] = avg + fuzz(width, scale);
}

fn calc_diamond_avg(grid: &mut Vec<Vec<f32>>, r: usize, c: usize, width: usize, scale: f32) {
	let mut count = 0;
	let mut avg = 0.0;
	if width <= c {
		avg += grid[r][c - width];
		count += 1;
	}
	if c + width < grid.len() {
		avg += grid[r][c + width];
		count += 1;
	}
	if width <= r {
		avg += grid[r - width][c];
		count += 1;
	}
	if r + width < grid.len() {
		avg += grid[r + width][c];
		count += 1;
	}
	
	grid[r][c] = avg / count as f32 + fuzz(width, scale);
}

fn square_step(grid: &mut Vec<Vec<f32>>, r: usize, c: usize, width: usize, scale: f32) {
	let half_width = width / 2;

	calc_diamond_avg(grid, r - half_width, c, half_width, scale);
	calc_diamond_avg(grid, r + half_width, c, half_width, scale);
	calc_diamond_avg(grid, r, c - half_width, half_width, scale);
	calc_diamond_avg(grid, r, c + half_width, half_width, scale);
}

fn diamond_sq(grid: &mut Vec<Vec<f32>>, r: usize, c: usize, width: usize, scale: f32) {
	diamond_step(grid, r, c, width, scale);
	let half_width = width / 2;
	square_step(grid, r + half_width, c + half_width, width, scale);

	if half_width == 1 {
		return;
	}

	let new_scale = scale * 1.95;
	diamond_sq(grid, r, c, half_width + 1, new_scale);
	diamond_sq(grid, r, c + half_width, half_width + 1, new_scale);
	diamond_sq(grid, r + half_width, c, half_width + 1, new_scale);
	diamond_sq(grid, r + half_width, c + half_width, half_width + 1, new_scale);
}

fn smooth_map(grid: &mut Vec<Vec<f32>>, width: usize) {
	for r in 0..width {
		for c in 0..width {
			let mut avg = grid[r][c];
			let mut count = 1;

			if r >= 1 {
				if c >= 1 {
					avg += grid[r - 1][c - 1];
					count += 1;
				}
				avg += grid[r - 1][c];
				count += 1;
				if c + 1 < width {
					avg += grid[r - 1][c + 1];
					count += 1;
				}
			}

			if c >= 1 {
				avg += grid[r][c - 1];
				count += 1;
			}
			if c + 1 < width {
				avg += grid[r][c + 1];
				count += 1;
			}

			if r + 1 < width {
				if c >= 1 {
					avg += grid[r + 1][c - 1];
					count += 1;
				}
				avg += grid[r + 1][c];
				count += 1;
				if c + 1 < width {
					avg += grid[r + 1][c + 1];
					count += 1;
				}
			}

			grid[r][c] = avg / count as f32;
		}
	}
}

fn warp_to_island(grid: &mut Vec<Vec<f32>>, width: usize, shift_y: f32) {
	for r in 0..width {
		for c in 0..width {
			let xd = c as f32 / (width as f32 - 1.0) * 2f32 - 1.0;
			let yd = r as f32 / (width as f32 - shift_y) * 2f32 - 1.0;
			let island_size = 0.96;
			grid[r][c] += island_size - f32::sqrt(xd*xd + yd*yd) * 3.0;
		}
	}
}

pub fn generate_island(width: usize) -> Vec<Vec<Tile>> {
	let mut grid = vec![vec![0.0f32; width]; width];

	grid[0][0] = rand::thread_rng().gen_range(0.0, 1.0);
	grid[0][width - 1] = rand::thread_rng().gen_range(0.0, 1.0);
	grid[width - 1][0] = rand::thread_rng().gen_range(0.0, 1.0);
	grid[width - 1][width - 1] = rand::thread_rng().gen_range(0.0, 1.0);

	let initial_scale = 1.0 / width as f32;
	diamond_sq(&mut grid, 0, 0, width, initial_scale);
	smooth_map(&mut grid, width);
	warp_to_island(&mut grid, width, 0.0);

	let mut map: Vec<Vec<Tile>> = Vec::new();
	for r in 0..width {
		let mut row = Vec::new();
		for c in 0..width {
			row.push(val_to_terrain(grid[r][c]));
		}
		map.push(row);
	}

	map
}

fn dump_cave(grid: &Vec<Vec<bool>>) {
	for row in grid {
		let mut s = String::from("");
		for val in row {
			if *val {
				s.push('#');
			} else {
				s.push('.');
			}
		}
		println!("{}", s);
	}
}

type DisjointSet = HashMap<(usize, usize), ((usize, usize), u32)>;

fn ds_find(ds: &mut DisjointSet, n: (usize, usize)) -> ((usize, usize), u32) {
	if !ds.contains_key(&n) {
		ds.insert((n.0, n.1), ((n.0, n.1), 1));
	}

	let res = ds.get(&n).unwrap();

	res.clone()
}

fn ds_union(ds: &mut DisjointSet, n1: (usize, usize), n2: (usize, usize)) {
	let s1 = ds_find(ds, n1);
	let s2 = ds_find(ds, n2);

	if s1.1 > s2.1 {
		let nv1 = ds.get_mut(&s1.0).unwrap();
		nv1.1 += s2.1;

		let nv2 = ds.get_mut(&s2.0).unwrap();
		nv2.0 = (n1.0, n1.1);
		nv2.1 = nv1.1;
	} else {
		let nv1 = ds.get_mut(&s2.0).unwrap();
		nv1.1 += s1.1;

		let nv2 = ds.get_mut(&s1.0).unwrap();
		nv2.0 = (n2.0, n2.1);
		nv2.1 = nv1.1;
	}
}

// The caves generated by the cellular automata method can end up disjoint --
// ie., smaller caves separated from each other. First, we need to group the
// floor squares together. Any set of 3 or fewer floors I'll just fill in.
//
// I'm going to treat squares as adjacent only if they are adjacent along the 
// 4 cardinal compass points.
// 
// To group then together, I'm going to use a disjoint set ADT.
fn cave_qa(grid: &mut Vec<Vec<bool>>, width: usize, depth: usize) {
	let mut ds: DisjointSet = HashMap::new();

	for r in 1..depth - 1 {
		for c in 1..width - 1 {
			if grid[r][c] { continue; }

			let f = ds_find(&mut ds, (r, c));
			println!("{:?}", f);
		}
	}
}

fn count_neighbouring_walls(grid: &Vec<Vec<bool>>, row: i32, col: i32, width: i32, depth: i32) -> u32 {
	let mut adj_walls = 0;

	for r in -1..2 {
		for c in -1..2 {
			let nr = row + r;
			let nc = col + c;
			if nr < 0 || nc < 0 || nr == depth || nc == width {
				adj_walls += 1;
			} else if !(nr == 0 && nc == 0) && grid[nr as usize][nc as usize] {
				adj_walls += 1;
			}
		}
	}	

	adj_walls
}

pub fn generate_cave(width: usize, depth: usize) {
	let mut grid = vec![vec![true; width]; depth];

	// Set some initial squares to be floors (false indidcates floor in our
	// initial grid)
	for r in 0..depth {
		for c in 0..width {
			let x: f64 = rand::thread_rng().gen();
			if x < 0.55 {
				grid[r][c] = false;
			}
		}
	}

	// We are using the 4-5 rule here (if a square has
	// 3 or fewer adjacents walls, it starves and becomes a floor,
	// if it has greater than 5 adj walls, it becomes a wall, otherwise
	// we leave it alone.
	//
	// One generation seems to generate nice enough maps!
	let mut next_gen: Vec<Vec<bool>> = Vec::new();
	next_gen = vec![vec![false; width]; depth];
	for r in 1..depth - 1 {
		for c in 1..width - 1 {
			let adj_walls = count_neighbouring_walls(&grid, r as i32, c as i32, width as i32, depth as i32);

			if adj_walls < 4 {
				next_gen[r][c] = false;
			} else if adj_walls > 5 {
				next_gen[r][c] = true;
			} else {
				next_gen[r][c] = grid[r][c];
			}
		}
	}

	grid = next_gen.clone();

	// set the border
	for c in 0..width {
		next_gen[0][c] = true;
		next_gen[depth - 1][c] = true;	
	}
	for r in 1..depth - 1 {
		next_gen[r][0] = true;
		next_gen[r][width - 1] = true;
	}

	cave_qa(&mut next_gen, width, depth);
	dump_cave(&next_gen);
}

pub fn generate_test_map() -> Vec<Vec<Tile>> {
	let mut map: Vec<Vec<Tile>> = Vec::new();

	let mut row = Vec::new();
	for r in 0..11 {
		row.push(Tile::Wall);
	}
	map.push(row);

	for r in 0..9 {
		let mut row = Vec::new();
		row.push(Tile::Wall);
		for c in 0..9 {
			row.push(Tile::Dirt);
		}
		row.push(Tile::Wall);
		map.push(row);
	}

	let mut row = Vec::new();
	for r in 0..11 {
		row.push(Tile::Wall);
	}
	map.push(row);

	map[8][9] = Tile::Wall;
	map[8][8] = Tile::Wall;
	map[8][7] = Tile::Wall;

	map[9][7] = Tile::Gate;

	map[2][7] = Tile::Wall;

	map[6][3] = Tile::Wall;
	map[6][5] = Tile::Wall;

	map[5][3] = Tile::Wall;
	map[5][5] = Tile::Wall;

	map[4][3] = Tile::Wall;
	map[4][5] = Tile::Wall;

	map[3][3] = Tile::Wall;
	map[3][4] = Tile::Wall;
	map[3][5] = Tile::Wall;

	map
}
