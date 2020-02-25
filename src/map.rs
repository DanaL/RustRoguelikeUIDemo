extern crate rand;

use rand::Rng;
use std::f32;

#[derive(Debug, Clone, Copy)]
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
}

fn val_to_terrain(val: f32) -> Tile {
	if val < -0.5 {
		return Tile::DeepWater;
	} else if val < -0.25 {
		return Tile::Water;
	} else if val < 0.0 {
		return Tile::Sand;	
	} else if val < 0.25 {
		return Tile::Dirt;
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

	let mut initial_scale = 1.0 / width as f32;
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
