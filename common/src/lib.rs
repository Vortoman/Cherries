use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Color {
    Red,
    Blue,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub color: Color,
}

impl User {
    pub fn new(name: String) -> User {
        User {
            name,
            color: Color::None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ColorSender {
    pub value: String,
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

#[derive(Debug)]
pub struct CellReadError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Neutral,
    Red,
    Blue,
}

impl Cell {
    pub fn to_color(&self) -> Color {
        match self {
            Cell::Empty => Color::None,
            Cell::Neutral => Color::None,
            Cell::Red => Color::Red,
            Cell::Blue => Color::Blue,
        }
    }
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
pub const N_NEUTRAL_BLOCKS: u32 = 100;

pub const WIDTH_CANVAS: u32 = (CELL_SIZE + 1) * WIDTH_UNIVERSE + 1;
pub const HEIGHT_CANVAS: u32 = (CELL_SIZE + 1) * HEIGHT_UNIVERSE + 1;

pub type Coords = (usize, usize);

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    timer: f64,

    pub red_player_connected: bool,
    pub blue_player_connected: bool,
}

enum CellWrapper<'a> {
    SelfManip,
    Extern(&'a mut Vec<(Cell, Coords)>),
}

impl Universe {
    pub fn _new_rand(cells: Vec<Cell>, n_empty: u32, n_neutral: u32) -> Self {
        Universe {
            cells,
            active_cells: vec![],
            width: WIDTH_UNIVERSE as usize,
            height: HEIGHT_UNIVERSE as usize,
            n_empty,
            n_neutral,
            n_red: 0,
            n_blue: 0,
            finished: false,
            timer: 3.,
            red_player_connected: false,
            blue_player_connected: false,
        }
    }

    pub fn new_empty() -> Universe {
        Universe {
            cells: Vec::from([Cell::Empty; (WIDTH_UNIVERSE * HEIGHT_UNIVERSE) as usize]),
            active_cells: vec![],
            width: WIDTH_UNIVERSE as usize,
            height: HEIGHT_UNIVERSE as usize,
            n_empty: (WIDTH_UNIVERSE * HEIGHT_UNIVERSE),
            n_neutral: 0,
            n_red: 0,
            n_blue: 0,
            finished: false,
            timer: 3.,
            red_player_connected: false,
            blue_player_connected: false,
        }
    }

    pub fn set_cell(&mut self, cell: &Cell, coords: Coords) -> Result<bool, CellReadError> {
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

    pub fn get_index(&self, coords: Coords) -> Result<usize, ()> {
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
                v.0 >= 0 && v.1 >= 0 && v.0 < WIDTH_UNIVERSE as i32 && v.1 < HEIGHT_UNIVERSE as i32
            })
            .map(|v| self.get_index((v.0 as usize, v.1 as usize)).unwrap())
            .collect()
    }

    fn _set_cell(
        &mut self,
        wrapped_vec: CellWrapper,
        cell: &Cell,
        coords: Coords,
    ) -> Result<bool, CellReadError> {
        let idx = self.get_index(coords).map_err(|_| CellReadError)?;
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
            _ => Err(CellReadError),
        }
    }
    pub fn get_cells(&self) -> Vec<Cell> {
        self.cells.clone()
    }

    pub fn get_cell_numbers(&self) -> (u32, u32, u32, u32) {
        (self.n_empty, self.n_red, self.n_blue, self.n_neutral)
    }

    pub fn get_timer(&self) -> f64 {
        self.timer
    }

    pub fn set_timer(&mut self, t: f64) {
        self.timer = t;
    }

    pub fn get_cell_numbers(&self) -> (u32, u32, u32, u32) {
        (self.n_empty, self.n_red, self.n_blue, self.n_neutral)
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
