use crate::board::Board;
use crate::rule::{Rule, State};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum WireWorldState {
    Void,
    Head,
    Tail,
    Wire,
}

impl std::default::Default for WireWorldState {
    fn default() -> Self {
        WireWorldState::Void
    }
}

impl State for WireWorldState {
    fn next(&self) -> Self {
        match *self {
            WireWorldState::Void => WireWorldState::Wire,
            WireWorldState::Wire => WireWorldState::Head,
            WireWorldState::Head => WireWorldState::Tail,
            WireWorldState::Tail => WireWorldState::Void,
        }
    }

    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        *self = match rng.gen_range(0..4) {
            0 => WireWorldState::Void,
            1 => WireWorldState::Head,
            2 => WireWorldState::Tail,
            3 => WireWorldState::Wire,
            _ => unreachable!(),
        }
    }

    fn clear(&mut self) {
        *self = WireWorldState::Void;
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(self, WireWorldState::Void, "Void");
        ui.radio_value(self, WireWorldState::Head, "Head");
        ui.radio_value(self, WireWorldState::Tail, "Tail");
        ui.radio_value(self, WireWorldState::Wire, "Wire");
    }
}

#[derive(Default)]
pub struct WireWorldRule {}

impl Rule for WireWorldRule {
    type CellState = WireWorldState;

    fn background(&self) -> egui::Color32 {
        egui::Color32::from_rgb(128, 128, 0)
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        match *st {
            WireWorldState::Void => egui::Color32::from_rgb(0, 0, 0),
            WireWorldState::Head => egui::Color32::from_rgb(0, 0, 255),
            WireWorldState::Tail => egui::Color32::from_rgb(255, 0, 0),
            WireWorldState::Wire => egui::Color32::from_rgb(255, 255, 0),
        }
    }

    fn update(&self, board: &mut Board<Self::CellState>) {
        for j in 0..board.height() {
            let yprev = if j == 0 { board.height() - 1 } else { j - 1 };
            let ynext = if j == board.height() - 1 { 0 } else { j + 1 };
            for i in 0..board.width() {
                match *board.cell_at(i, j) {
                    WireWorldState::Void => {
                        *board.bufcell_at_mut(i, j) = WireWorldState::Void;
                    }
                    WireWorldState::Head => {
                        *board.bufcell_at_mut(i, j) = WireWorldState::Tail;
                    }
                    WireWorldState::Tail => {
                        *board.bufcell_at_mut(i, j) = WireWorldState::Wire;
                    }
                    WireWorldState::Wire => {
                        let xprev = if i == 0 { board.width() - 1 } else { i - 1 };
                        let xnext = if i == board.width() - 1 { 0 } else { i + 1 };
                        let mut nheads = 0;
                        for ny in [yprev, j, ynext] {
                            for nx in [xprev, i, xnext] {
                                if *board.cell_at(nx, ny) == WireWorldState::Head {
                                    nheads += 1;
                                }
                            }
                        }
                        if nheads == 1 || nheads == 2 {
                            *board.bufcell_at_mut(i, j) = WireWorldState::Head;
                        } else {
                            *board.bufcell_at_mut(i, j) = WireWorldState::Wire;
                        }
                    }
                }
            }
        }
        std::mem::swap(&mut board.chunks, &mut board.buffer);
    }
    // draw rule-specific part of UI
}
