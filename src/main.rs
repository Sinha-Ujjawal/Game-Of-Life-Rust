// Stolen from https://github.com/tsoding/carrotson/blob/master/carrotson.rs
struct LCG {
    state: u64,
}

impl LCG {
    fn from_sys_timestamp() -> Self {
        Self::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64,
        )
    }

    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn random_u32(&mut self) -> u32 {
        // Stolen from https://en.wikipedia.org/wiki/Linear_congruential_generator
        // Using the values of MMIX by Donald Knuth
        const RAND_A: u64 = 6364136223846793005;
        const RAND_C: u64 = 1442695040888963407;
        (self.state, _) = self.state.overflowing_mul(RAND_A);
        (self.state, _) = self.state.overflowing_add(RAND_C);
        return (self.state >> 32) as u32;
    }
}

impl Iterator for LCG {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.random_u32())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Coord {
    x: i16,
    y: i16,
}

impl Coord {
    fn step(&self, velx: i16, vely: i16) -> Coord {
        Self {
            x: self.x + velx,
            y: self.y + vely,
        }
    }

    fn neighbors(&self) -> Vec<Coord> {
        vec![
            self.step(-1, -1),
            self.step(0, -1),
            self.step(1, -1),
            self.step(-1, 0),
            self.step(1, 0),
            self.step(-1, 1),
            self.step(0, 1),
            self.step(1, 1),
        ]
    }

    fn wrap(&self, width: u8, height: u8) -> Self {
        Self {
            x: self.x.rem_euclid(width as i16),
            y: self.y.rem_euclid(height as i16),
        }
    }

    fn index_in(&self, width: u8, height: u8) -> usize {
        let coord = self.wrap(width, height);
        (coord.y as usize) * (width as usize) + (coord.x as usize)
    }

    fn random_coords(rng: &mut LCG, take: usize) -> Vec<Self> {
        (0..usize::MAX)
            .map(|_| {
                let x = rng.next().unwrap();
                let y = rng.next().unwrap();
                Self {
                    x: x as i16,
                    y: y as i16,
                }
            })
            .take(take)
            .collect()
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum CellStatus {
    Alive,
    Dead,
}

#[derive(Debug)]
struct GOL {
    width: u8,
    height: u8,
    grid: Vec<CellStatus>,
}

impl GOL {
    fn is_alive(&self, coord: &Coord) -> bool {
        self.grid[coord.index_in(self.width, self.height)] == CellStatus::Alive
    }

    fn step(&mut self) {
        let mut next_grid = self.grid.clone();
        for y in 0..self.height as i16 {
            for x in 0..self.width as i16 {
                let coord = Coord { x, y };
                let idx = coord.index_in(self.width, self.height);
                let num_live_neighbors = coord
                    .neighbors()
                    .into_iter()
                    .filter(|coord| self.is_alive(&coord))
                    .count();
                next_grid[idx] = match (&self.grid[idx], num_live_neighbors) {
                    // Rule 1. Any live cell with fewer than 2 live neighbors dies, as if casued by underpopulation
                    (CellStatus::Alive, x) if x < 2 => CellStatus::Dead,
                    // Rule 2. Any live cell with 2 or 3 live neighbors get's to survive to the next generation
                    (CellStatus::Alive, 2) | (CellStatus::Alive, 3) => CellStatus::Alive,
                    // Rule 3. Any live cell with more than 3 live neighbors dies, as if caused by overpopulation
                    (CellStatus::Alive, x) if x > 3 => CellStatus::Dead,
                    // Rule 4. Any dead cell with exactly three neighbors becomes alive, as if by reproduction
                    (CellStatus::Dead, 3) => CellStatus::Alive,
                    // All other cells remain in the same state
                    (otherwise, _) => otherwise.clone(),
                };
            }
        }
        self.grid = next_grid;
    }

    fn print_to_console(&self) {
        print!(" ");
        println!("{}", "# ".repeat(self.width as usize + 1));
        for y in 0..self.height as i16 {
            print!("# ");
            for x in 0..self.width as i16 {
                let coord = Coord { x, y };
                let idx = coord.index_in(self.width, self.height);
                if self.grid[idx] == CellStatus::Alive {
                    print!("o ");
                } else {
                    print!("  ");
                }
            }
            println!("#");
        }
        print!(" ");
        println!("{}", "# ".repeat(self.width as usize + 1));
    }

    fn clear_console() {
        print!("\x1B[2J\x1B[1;1H");
    }

    fn simulate(&mut self, speed: std::time::Duration) {
        print!("\x1b[?25l");
        loop {
            self.print_to_console();
            self.step();
            std::thread::sleep(speed);
            GOL::clear_console();
        }
    }

    #[allow(dead_code)]
    fn glider_pattern(width: u8, height: u8) -> Self {
        GOL::from_iter(
            width,
            height,
            vec![
                Coord { x: 0, y: 0 },
                Coord { x: 1, y: 1 },
                Coord { x: 2, y: 1 },
                Coord { x: 0, y: 2 },
                Coord { x: 1, y: 2 },
            ]
            .into_iter(),
        )
    }

    fn from_iter(width: u8, height: u8, live_coords: impl Iterator<Item = Coord>) -> Self {
        let mut grid: Vec<CellStatus> = (0..(width as usize * height as usize))
            .into_iter()
            .map(|_| CellStatus::Dead)
            .collect();

        live_coords.for_each(|coord| {
            let idx = coord.index_in(width, height);
            grid[idx] = CellStatus::Alive;
        });

        GOL {
            width,
            height,
            grid,
        }
    }
}

fn main() {
    let mut rng = LCG::from_sys_timestamp();
    let mut gol = GOL::from_iter(15, 15, Coord::random_coords(&mut rng, 100).into_iter());
    // let mut gol = GOL::glider_pattern(15, 15);
    // println!("{:?}", gol);
    gol.simulate(std::time::Duration::from_millis(100));
}
