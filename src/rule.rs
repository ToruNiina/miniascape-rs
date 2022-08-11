use rand::Rng;

/// State of a cell.
///
/// Represents the current state of a cell. To initialize the board, it requires
/// `Clone`. To clear the board, it requires `Default`.
/// The update rule is implemented in `Rule` trait.
pub trait State: Clone + Default + std::fmt::Debug {
    /// Generate UI to inspect and modify the cell state.
    fn inspect(&mut self, ui: &mut egui::Ui, buf: &mut String);
}

/// Rule of the cellular automaton.
pub trait Rule<const N: usize, Neighborhood: Neighbors<N>>: Default {
    /// Corresponding cell state.
    type CellState: State;

    /// Background color.
    fn background(&self) -> egui::Color32;

    /// Color of a cell.
    fn color(&self, st: &Self::CellState) -> anyhow::Result<egui::Color32>;

    fn default_state(&self) -> anyhow::Result<Self::CellState>;

    fn randomize<R: Rng>(&self, rng: &mut R) -> anyhow::Result<Self::CellState>;

    /// The next state. It will be used to change cell state from GUI
    fn next(&self, st: Self::CellState) -> anyhow::Result<Self::CellState>;

    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> [(usize, usize); N] {
        Neighborhood::neighbors(x, y, w, h)
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbors: impl Iterator<Item = Self::CellState>,
    ) -> anyhow::Result<Self::CellState>;

    fn iteration_per_step(&self) -> u32 {
        1
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
}

pub trait Neighbors<const N: usize>: Default {
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> [(usize, usize); N];
}

#[derive(Default)]
pub struct VonNeumannNeighborhood {}
#[derive(Default)]
pub struct MooreNeighborhood {}
#[derive(Default)]
pub struct HexGridNeighborhood {}

impl Neighbors<4> for VonNeumannNeighborhood {
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> [(usize, usize); 4] {
        let xprev = (if x == 0 { w - 1 } else { x - 1 }) as usize;
        let xnext = (if x == w - 1 { 0 } else { x + 1 }) as usize;
        let yprev = (if y == 0 { h - 1 } else { y - 1 }) as usize;
        let ynext = (if y == h - 1 { 0 } else { y + 1 }) as usize;
        let x = x as usize;
        let y = y as usize;
        [(x, yprev), (xnext, y), (xprev, y), (x, ynext)]
    }
}
impl Neighbors<8> for MooreNeighborhood {
    #[rustfmt::skip]
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> [(usize, usize); 8] {
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
impl Neighbors<6> for HexGridNeighborhood {
    #[rustfmt::skip]
    fn neighbors(x: isize, y: isize, w: isize, h: isize) -> [(usize, usize); 6] {
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
}
