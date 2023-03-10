// use rand::Rng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: String) -> User {
        User { name }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserList {
    pub users: Vec<User>,
    pub n_users: u32,
}

impl UserList {
    pub fn new() -> UserList {
        UserList {
            users: vec![],
            n_users: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Neutral,
    Red,
    Blue,
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}

pub const CELL_SIZE: u32 = 15;
pub const GRID_COLOR: &str = "#CCCC";
pub const EMPTY_COLOR: &str = "#FFFFFF";
pub const WALL_COLOR: &str = "#000000";
pub const BLUE_COLOR: &str = "#0000FF";
pub const RED_COLOR: &str = "#FF0000";

pub const WIDTH_UNIVERSE: u32 = 32;
pub const HEIGHT_UNIVERSE: u32 = 32;
pub const N_NEUTRAL_BLOCKS: u32 = 50;

pub const WIDTH_CANVAS: u32 = (CELL_SIZE + 1) * WIDTH_UNIVERSE as u32 + 1;
pub const HEIGHT_CANVAS: u32 = (CELL_SIZE + 1) * HEIGHT_UNIVERSE as u32 + 1;

type Coords = (usize, usize);

pub struct Universe {
    cells: Vec<Cell>,
    active_cells: Vec<(Cell, Coords)>,
    width: usize,
    height: usize,
    n_empty: u32,
    n_neutral: u32,
    n_red: u32,
    n_blue: u32,
    finished: bool,
}

enum CellWrapper<'a> {
    SelfManip,
    Extern(&'a mut Vec<(Cell, Coords)>),
}

impl Universe {
    pub fn new_empty() -> Universe {
        Universe {
            cells: Vec::from([Cell::Empty; (WIDTH_UNIVERSE * HEIGHT_UNIVERSE) as usize]),
            active_cells: vec![],
            width: WIDTH_UNIVERSE as usize,
            height: HEIGHT_UNIVERSE as usize,
            n_empty: (WIDTH_UNIVERSE * HEIGHT_UNIVERSE) as u32,
            n_neutral: 0,
            n_red: 0,
            n_blue: 0,
            finished: false,
        }
    }

    pub fn new_rand() -> Universe {
        let mut uni = Self::new_empty();
        for _ in 0..N_NEUTRAL_BLOCKS {
            // let idx = rand::random::<usize>() % uni.cells.len();
            let idx = 1;
            if uni.cells[idx] == Cell::Empty {
                uni.cells[idx] = Cell::Neutral;
                uni.n_empty -= 1;
                uni.n_neutral += 1;
            }
        }
        uni
    }

    pub fn set_cell(&mut self, cell: &Cell, coords: Coords) -> Result<bool, ()> {
        self._set_cell(CellWrapper::SelfManip, cell, coords)
    }

    pub fn evolve(&mut self) {
        let mut next_cells = vec![];
        while let Some(cell) = self.active_cells.pop() {
            for neighbour_idx in self.get_neighbours(cell.1) {
                if self.cells[neighbour_idx] == Cell::Empty {
                    if let Ok(_) = self._set_cell(
                        CellWrapper::Extern(&mut next_cells),
                        &cell.0.clone(),
                        self.get_coords(neighbour_idx),
                    ) {};
                }
            }
        }
        self.active_cells = next_cells;
        if self.n_empty == 0 {
            self.finished = true;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_index(&self, coords: Coords) -> Result<usize, ()> {
        let out = coords.0 + coords.1 * self.width;
        if out > self.cells.len() {
            Err(())
        } else {
            Ok(out)
        }
    }

    fn get_coords(&self, idx: usize) -> (usize, usize) {
        ((idx % self.width), (idx / self.height))
    }

    fn get_neighbours(&self, coords: Coords) -> Vec<usize> {
        let coords = (coords.0 as i32, coords.1 as i32);
        let directions = [(0, 1), (1, 0), (-1, 0), (0, -1)];
        directions
            .map(|d| (coords.0 + d.0, coords.1 + d.1))
            .into_iter()
            .filter(|v| {
                v.0 > 0 && v.1 > 0 && v.0 < WIDTH_UNIVERSE as i32 && v.1 < HEIGHT_UNIVERSE as i32
            })
            .map(|v| self.get_index((v.0 as usize, v.1 as usize)).unwrap())
            .collect()
    }

    fn _set_cell(
        &mut self,
        wrapped_vec: CellWrapper,
        cell: &Cell,
        coords: Coords,
    ) -> Result<bool, ()> {
        let idx = self.get_index(coords)?;
        let cell_vec = match wrapped_vec {
            CellWrapper::SelfManip => &mut self.active_cells,
            CellWrapper::Extern(v) => v,
        };
        match *cell {
            Cell::Red | Cell::Blue | Cell::Neutral => {
                if self.cells[idx] == Cell::Empty {
                    self.cells[idx] = *cell;
                    cell_vec.push((*cell, coords));
                    self.n_empty -= 1;
                    if *cell == Cell::Red {
                        self.n_red += 1
                    }
                    if *cell == Cell::Blue {
                        self.n_blue += 1
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Self { x, y }
    }
}
