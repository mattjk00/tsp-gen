use rand::Rng;
use rand::rngs::ThreadRng;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path as fPath;

struct CityMap {
	count:i64,
	edges:Vec<i64>
}

struct Path {
	verts:Vec<i64>,
	score:i64	
}

struct Log {
	file:File,
	logs:Vec<Vec<i64>>
}

struct Population {
	paths:Vec<Path>,
	size:i64,
	max_gens:i64,
	gen_count:i64,
	cities:CityMap,
	rng:ThreadRng,
	logger:Log
}


impl Log {
	fn new(file_name:String) -> Log {

		let path = fPath::new(&file_name);

		let mut file = match File::create(&path) {
			Err(e) => {
				println!("Could not create file: {}", file_name);

				let mut f = match File::open(&path) {
					Err(why) => panic!("couldn't open {}: {}", file_name, why),
					Ok(f) => f,
				};
				f
			},
			Ok(file) => file,
		};
		Log { file:file, logs:vec![] }
	}

	fn put(&mut self, v:Vec<i64>) {
		self.logs.push(v);
	}

	fn put_scores(&mut self, s:&Vec<Path>) {
		let mut gen_scores:Vec<i64> = vec![];

		for p in s {
			gen_scores.push(p.score);
		}
		self.logs.push(gen_scores);
	}

	fn save_to_file(&mut self) {
		let mut king = "".to_owned();

		for v in &self.logs {
			let mut vals:Vec<String> = vec![];
			for l in v {
				vals.push(l.to_string());
			}
			let joined = vals.join(" ");
			king.push_str(&joined);
			king.push_str("\n");
		}
		
		match self.file.write_all(king.as_bytes()) {
			Err(e) => panic!("Failed to write log file! {}", e),
			Ok(_) => println!("Log file written.")
		};
	}
}

fn print_vec(v:&Vec<i64>) {
	for x in v {
		print!("{},", x);
	}
	println!();
}

impl CityMap {
	fn distance(&self, city1:i64, city2:i64) -> i64 {
		let index:usize = (city1 * self.count + city2) as usize;
		self.edges[index]
	}
}

impl Population {
	fn new(size:i64, maxg:i64, cs:CityMap, rng:ThreadRng, log:Log) -> Population {
		Population { paths:vec![], size:size, 
				max_gens:maxg, gen_count:0,
				cities:cs, rng:rng, logger:log }
	}
	fn init_random(&mut self) {
		//let i = rng.gen_range(0..10);
		
		for i in 0..self.size {
			let mut sp = self.straight_path();
			let mut pverts:Vec<i64> = vec![];
			while sp.len() > 0 {
				let index = self.rng.gen_range(0..sp.len()) as usize;
				pverts.push(sp.remove(index));
			}
			pverts.push(pverts[0]);
			let mut path = Path::new_with_verts(pverts);
			let score  = self.fitness(&path);
			path.score = score;
			self.paths.push(path);	
		}
		self.logger.put_scores(&self.paths);
	}

	fn next_generation(&mut self) {
		let best_indices = self.best_two();
		let best_a = self.copy_vec(&self.paths[best_indices.0].verts);
		let best_b = self.copy_vec(&self.paths[best_indices.1].verts);
		let mut new_paths = vec![];

		let mut best_path = Path::new_with_verts(self.copy_vec(&best_a));
		best_path.score = self.fitness(&best_path);
		new_paths.push(best_path);

		let mut best_pathb = Path::new_with_verts(self.copy_vec(&best_b));
		best_pathb.score = self.fitness(&best_pathb);
		new_paths.push(best_pathb);

		let half_size = (self.size-2) / 2;
		
		for i in 0..half_size {
			let child_verts = self.mate(&best_a, &best_b);
			let mut path = Path::new_with_verts(child_verts);
			path.score = self.fitness(&path);
			new_paths.push(path);
		}
		for i in 0..half_size {
			let child_verts = self.mate(&best_b, &best_a);
			let mut path = Path::new_with_verts(child_verts);
			path.score = self.fitness(&path);
			new_paths.push(path);
		}

		self.paths = new_paths;
		self.logger.put_scores(&self.paths);
	}

	fn copy_vec(&self, a:&Vec<i64>) -> Vec<i64> {
		let mut copy = vec![];
		for v in a {
			copy.push(*v);
		}
		copy
	}

	fn mate(&mut self, a:&Vec<i64>, b:&Vec<i64>) -> Vec<i64> {
		let xstart = self.rng.gen_range(1.. a.len()-1);
		let xend = self.rng.gen_range(xstart..a.len()-1);
		let mut c:Vec<i64> = vec![-1; a.len()];
		c[0] = a[0];
		
		// fill in the crossover from a
		for i in 1..a.len()-1 {
			if i >= xstart && i <= xend {
				c[i] = a[i];
			}
		}
		c[a.len()-1]=c[0];
		
		let mut bclone = b.clone();
		for i in 0..bclone.len() {
			if c.contains(&b[i]) {
				bclone[i] = -1;
			}
		}
		
		let mut extra = vec![];
		for i in 0..bclone.len() {
			if bclone[i] != -1 {
				extra.push(bclone[i]);
			}
		}
		
		
		for i in 0..c.len() {
			if c[i] == -1 {
				c[i] = extra.remove(0);
			}
		}
		
		// mutation
		let sa = self.rng.gen_range(1..a.len()-1); // swap a
		let sb = self.rng.gen_range(1..a.len()-1); // swap b
		let save = c[sa];
		c[sa] = c[sb];
		c[sb] = save;
		c
	}

	fn best_two(&self) -> (usize, usize) {
		let mut a:usize = 0;
		let mut b:usize = 0;
		for i in 0..self.paths.len() {
			if self.paths[i].score < self.paths[a].score {
				b = a;
				a = i;
			}
			else if self.paths[i].score < self.paths[b].score {
				b = i;
			}
		}
		(a, b)
	}

	fn straight_path(&self) -> Vec<i64> {
		let mut path:Vec<i64> = vec![];
		for i in 0..self.cities.count {
			path.push(i as i64);
		}
		path
	}
	
	fn fitness(&self, path:&Path) -> i64 {
		let mut sum = 0i64;
		for i in 0..(path.verts.len()-1) {
			sum += self.cities.distance(path.verts[i], path.verts[i+1]);
		}
		sum
	}

	fn print(&self) {
		println!("DNA\t\tScore");
		for p in 0..self.paths.len() {
			for c in 0..self.paths[p].verts.len() {
				print!("{}", self.paths[p].verts[c]);
			}
			println!("\t{}", self.paths[p].score);
		}
	}

	fn save_logs(&mut self) {
		self.logger.save_to_file();
	}
}

impl Path {
	fn new() -> Path {
		Path { verts:vec![], score:-1 }
	}
	fn new_with_verts(v:Vec<i64>) -> Path {
		let mut path = Path::new();
		path.set_vertices(v);
		path
	}
	fn set_vertices(&mut self, v:Vec<i64>) {
		self.verts = v;
	}
}

fn main() {
	let mut logger = Log::new(String::from("log.txt"));
	//logger.put(vec![1, 2, 3, 4, 5]);
	//logger.save_to_file();
	let mut rng = rand::thread_rng();
	let map = CityMap {
		count:6,
		edges:vec![	0, 25, 40, 100, 40, 32,
			   		25, 0, 30, 165, 190, 90,
			   		40, 30, 0, 45, 85, 215,
			   		100, 165, 45, 0, 15, 120,
			   		40, 190, 85, 15, 0, 75,
			   		32, 90, 215, 15, 75, 0]
	};
	println!("{}", map.distance(0, 1));
	let mut pop = Population::new(10, 10, map, rng, logger);
	pop.init_random();
	//pop.print();
	for _i in 0..499 {
		pop.next_generation();
		//pop.print();
	}
	pop.save_logs();
}
