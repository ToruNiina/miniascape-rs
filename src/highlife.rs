use crate::app::{Board, Rule};
use crate::conway::LifeGameState;

pub struct HighLifeRule {}

impl Rule for HighLifeRule {
    type CellState = LifeGameState;

    fn background() -> egui::Color32 {
        egui::Color32::from_rgb(0, 128, 0)
    }

    fn color(st: &Self::CellState) -> egui::Color32 {
        if *st == LifeGameState::Dead {
            egui::Color32::from_rgb(0, 0, 0)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        }
    }

    fn update(board: &mut Board<Self::CellState>) {
        for j in 0..board.height() {
            let yprev = if j == 0 { board.height() - 1 } else { j - 1 };
            let ynext = if j == board.height() - 1 { 0 } else { j + 1 };
            for i in 0..board.width() {
                let xprev = if i == 0 { board.width() - 1 } else { i - 1 };
                let xnext = if i == board.width() - 1 { 0 } else { i + 1 };
                let mut nalive = 0;
                for ny in [yprev, j, ynext] {
                    for nx in [xprev, i, xnext] {
                        if *board.cell_at(nx, ny) == LifeGameState::Alive {
                            nalive += 1;
                        }
                    }
                }
                let self_is_alive = *board.cell_at(i, j) == LifeGameState::Alive;

                let buf = board.bufcell_at_mut(i, j);
                // 23/36
                *buf = if self_is_alive {
                    if nalive == 2+1 || nalive == 3+1 {
                        LifeGameState::Alive
                    } else {
                        LifeGameState::Dead
                    }
                } else {
                    if nalive == 3 || nalive == 6 {
                        LifeGameState::Alive
                    } else {
                        LifeGameState::Dead
                    }
                };
            }
        }
        std::mem::swap(&mut board.chunks, &mut board.buffer);
    }
    // draw rule-specific part of UI
}
