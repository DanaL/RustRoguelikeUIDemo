use std::collections::HashMap;
use std::collections::VecDeque;

use crate::map;

fn manhattan_d(ax: usize, ay: usize, bx: usize, by: usize) -> usize {
	((ax as i32 - bx as i32).abs() + (ay as i32 - by as i32).abs()) as usize	
}

pub fn find_path(map: &Vec<Vec<map::Tile>>, start_r: usize, start_c: usize, end_r: usize, end_c: usize) {
	let mut visited = HashMap::new();
	visited.insert((start_c, start_c), 0);
	let mut open = VecDeque::new();
	open.push_back((start_r, start_c));

	while open.len() > 0 {
		let current = open.pop_front().unwrap();

		for r in -1..2 {
			for c in -1..2 {
				if r == 0 && c == 0 { continue; }
	
				let nr = current.0 as i32 + r;
				let nc = current.1 as i32 + c;
				let successor = (nr as usize, nc as usize);
			
				if !map::is_passable(map[successor.0][successor.1]) {
					continue;
				}
	
				if manhattan_d(successor.0, successor. 1, end_r, end_c) <= 1 {
					println!("goal found!");
					return;
				}

				let g = 1 + visited[&(current.0, current.1)];
				let h = successor.0 - end_r + successor.1 - end_c;
				if h < 0 {
					h *= -1;
				}
					
			}
		}	
	}
}
