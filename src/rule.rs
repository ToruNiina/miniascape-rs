use rand::Rng;

/// State of a cell.
///
/// Represents the current state of a cell. To initialize the board, it requires
/// `Clone`. To clear the board, it requires `Default`.
/// The update rule is implemented in `Rule` trait.
pub trait State: Clone + Copy + Default {
    /// The next state. It will be used to change cell state from GUI
    fn next(&self) -> Self;

    /// Randomize the cell state. It will be used to initialize the board.
    fn randomize<R: Rng>(&mut self, rng: &mut R);

    /// Clear the current state.
    fn clear(&mut self);

    /// Generate UI to inspect and modify the cell state.
    fn inspect(&mut self, ui: &mut egui::Ui);
}

/// Rule of the cellular automaton.
pub trait Rule<const N: usize, Neighborhood: Neighbors<N>>: Default {
    /// Corresponding cell state.
    type CellState: State;

    /// Background color.
    fn background(&self) -> egui::Color32;

    /// Color of a cell.
    fn color(&self, st: &Self::CellState) -> egui::Color32;

    fn neighbors(x: isize, y: isize) -> [(isize, isize); N] {
        Neighborhood::neighbors(x, y)
    }

    fn update(&self, center: Self::CellState, neighbors: impl Iterator<Item = Self::CellState>) -> Self::CellState;

    fn iteration_per_step(&self) -> u32 {
        1
    }

    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait Neighbors<const N: usize>: Default {
    fn neighbors(x: isize, y: isize) -> [(isize, isize); N];
}

#[derive(Default)]
pub struct VonNeumannNeighborhood{}
#[derive(Default)]
pub struct MooreNeighborhood{}
#[derive(Default)]
pub struct HexGridNeighborhood{}

impl Neighbors<4> for VonNeumannNeighborhood {
    fn neighbors(x: isize, y: isize) -> [(isize, isize); 4] {
        [(x, y-1), (x-1, y), (x+1, y), (x, y+1)]
    }
}
impl Neighbors<8> for MooreNeighborhood {
    #[rustfmt::skip]
    fn neighbors(x: isize, y: isize) -> [(isize, isize); 8] {
        [(x-1, y-1), (x, y-1), (x+1, y-1),
         (x-1, y  ),           (x+1, y  ),
         (x-1, y+1), (x, y+1), (x+1, y+1)]
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
    fn neighbors(x: isize, y: isize) -> [(isize, isize); 6] {
        if x % 2 == 0 {
            [(x-1, y-1), (x, y-1),
             (x-1, y  ), (x+1, y),
             (x-1, y+1), (x, y+1)]
        } else {
            [(x, y-1), (x+1, y-1),
             (x-1, y), (x+1, y  ),
             (x, y+1), (x+1, y+1)]
        }
    }
}
