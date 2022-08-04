use crate::board::Board;
use rand::Rng;

/// State of a cell.
///
/// Represents the current state of a cell. To initialize the board, it requires
/// `Clone`. To clear the board, it requires `Default`.
/// The update rule is implemented in `Rule` trait.
pub trait State: Clone + Default {
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
pub trait Rule: Default {
    /// Corresponding cell state.
    type CellState: State;

    /// Background color.
    fn background(&self) -> egui::Color32;

    /// Color of a cell.
    fn color(&self, st: &Self::CellState) -> egui::Color32;

    /// Update the whole board
    fn update(&self, board: &mut Board<Self::CellState>);

    fn ui(&mut self, ui: &mut egui::Ui);
}
