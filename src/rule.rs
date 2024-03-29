use crate::board::ClipBoard;
use rand::Rng;

/// State of a cell.
///
/// Represents the current state of a cell.
///
/// To initialize the board, it requires `Clone` and `Default`.
/// But before any operation, the state will be cleared by using `Rule::default_state`.
///
/// Most of the operations are provided in `Rule` trait.
///
pub trait State: Clone + Default + std::fmt::Debug {
    /// Generate UI to inspect and modify the cell state.
    fn inspect(&mut self, ui: &mut egui::Ui, buf: &mut String);
}

/// Rule of the cellular automaton.
///
/// It contains rule of update/clear/randomize, visualization, and UI related functions.
///
/// Since `miniascape` supports `DynamicRule` that takes rhai script as the update rule,
/// most of the functions *can fail*. For example, it fails if the rhai script contains
/// a syntax error.
///
pub trait Rule: Default {
    /// Corresponding cell state.
    type CellState: State;

    type Neighborhood;

    /// Background color.
    fn background(&self) -> egui::Color32;

    /// Color of a cell.
    fn color(&self, st: &Self::CellState) -> anyhow::Result<egui::Color32>;

    /// the default cell state. When a board is cleared, all the cells have this value.
    fn default_state(&self) -> anyhow::Result<Self::CellState>;

    /// randomize cell state.
    fn randomize<R: Rng>(&self, rng: &mut R) -> anyhow::Result<Self::CellState>;

    /// The next state. It will be used to change the state of a cell from GUI
    fn next(&self, st: Self::CellState) -> anyhow::Result<Self::CellState>;

    /// Update the center cell using the neighboring cells.
    fn update(
        &self,
        center: Self::CellState,
        neighbors: impl Iterator<Item = Self::CellState>,
    ) -> anyhow::Result<Self::CellState>;

    /// The number of updates in one step. Normally 1.
    /// This *step* means update of a window.
    fn iteration_per_step(&self) -> u32 {
        1
    }

    fn library(&self) -> Vec<(String, ClipBoard<Self::CellState>)> {
        Vec::new()
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        on_side_panel: bool,
    ) -> anyhow::Result<()>;
}

/// Index of neighboring cells.
pub trait Neighbors: Default {
    type Neighborhood: IntoIterator<Item = (usize, usize)>;
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> Self::Neighborhood;
    fn num_neighbors() -> usize;
}

/// Von-Neumann Neighborhood. Up, Down, Left, Right cells are the neighbors.
#[derive(Default)]
pub struct VonNeumannNeighborhood {}

/// Moore Neighborhood. 8 neighboring cells in a square grid are the neighbors.
#[derive(Default)]
pub struct MooreNeighborhood {}

/// Neighborhood on a hexagonal grid.
#[derive(Default)]
pub struct HexGridNeighborhood {}

impl Neighbors for VonNeumannNeighborhood {
    type Neighborhood = [(usize, usize); 4];
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> Self::Neighborhood {
        let xprev = (if x == 0 { w - 1 } else { x - 1 }) as usize;
        let xnext = (if x == w - 1 { 0 } else { x + 1 }) as usize;
        let yprev = (if y == 0 { h - 1 } else { y - 1 }) as usize;
        let ynext = (if y == h - 1 { 0 } else { y + 1 }) as usize;
        let x = x as usize;
        let y = y as usize;
        [(x, yprev), (xnext, y), (xprev, y), (x, ynext)]
    }

    fn num_neighbors() -> usize {
        4
    }
}
impl Neighbors for MooreNeighborhood {
    type Neighborhood = [(usize, usize); 8];

    #[rustfmt::skip]
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> Self::Neighborhood {
        let xprev = (if x == 0 { w - 1 } else { x - 1 }) as usize;
        let xnext = (if x == w - 1 { 0 } else { x + 1 }) as usize;
        let yprev = (if y == 0 { h - 1 } else { y - 1 }) as usize;
        let ynext = (if y == h - 1 { 0 } else { y + 1 }) as usize;
        let x = x as usize;
        let y = y as usize;
        [(xprev, yprev), (x, yprev), (xnext, yprev),
         (xprev, y    ),             (xnext, y    ),
         (xprev, ynext), (x, ynext), (xnext, ynext)]
    }

    fn num_neighbors() -> usize {
        8
    }
}

// square-shaped hexgrid indexing
//
//   .-- x
//   |
//   y   .'.   .'.   .'.   .'.   .'.   .'.   .'.
//     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.
// 0   | 0,0 | 1,0 | 2,0 | 3,0 | 4,0 |     |N-1,0|
//     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
//       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
// 1      | 0,1 | 1,1 | 2,1 | 3,1 |     |     |N-1,1|
//       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
//     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
// 2   | 0,2 | 1,2 | 2,2 | 3,2 | 4,2 |     |N-1,2|
//     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
//       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
// 3      | 0,3 | 1,3 | 2,3 | 3,3 |     |     |     |
//       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
//     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
// 4   | 0,4 |     |     |     |     |     |     |
//     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
//       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
// 5      | 0,5 |     |     |     |     |     |     |
//       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
//     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
// 6   |     |     |     |     |     |     |     |
//     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
//       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
// 7      |N-1,0|     |     |     |     |     |N-1,N-1|
//        '.   .'.   .'.   .'.   .'.   .'.   .'.   .'
//          `.'   `.'   `.'   `.'   `.'   `.'   `.'
//
impl Neighbors for HexGridNeighborhood {
    type Neighborhood = [(usize, usize); 6];
    #[rustfmt::skip]
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> Self::Neighborhood {
        let xprev = (if x == 0 { w - 1 } else { x - 1 }) as usize;
        let xnext = (if x == w - 1 { 0 } else { x + 1 }) as usize;
        let yprev = (if y == 0 { h - 1 } else { y - 1 }) as usize;
        let ynext = (if y == h - 1 { 0 } else { y + 1 }) as usize;
        let x = x as usize;
        let y = y as usize;

        if y % 2 == 0 {
            [(xprev, yprev), (x,     yprev),
             (xprev, y    ), (xnext, y),
             (xprev, ynext), (x,     ynext)]
        } else {
            [(x, yprev), (xnext, yprev),
             (xprev, y), (xnext, y),
             (x, ynext), (xnext, ynext)]
        }
    }
    fn num_neighbors() -> usize {
        6
    }
}
