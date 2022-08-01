use crate::app::{Board, Rule};
use crate::conway::LifeGameState;

use arrayvec::ArrayVec;

pub struct GeneralizedLifeGameRule {
    alive: ArrayVec<u32, 9>, // number of neighboring cells is in [0, 8]
    birth: ArrayVec<u32, 9>,
}

impl Default for GeneralizedLifeGameRule {
    fn default() -> Self {
        Self {
            alive: (&[2_u32, 3_u32] as &[_]).try_into().unwrap(),
            birth: (&[3_u32] as &[_]).try_into().unwrap(),
        }
    }
}

impl GeneralizedLifeGameRule {
    pub fn new(alive: Vec<u32>, birth: Vec<u32>) -> Self {
        Self {
            alive: ArrayVec::from_iter(alive.into_iter()),
            birth: ArrayVec::from_iter(birth.into_iter()),
        }
    }
}

impl Rule for GeneralizedLifeGameRule {
    type CellState = LifeGameState;

    fn background(&self) -> egui::Color32 {
        egui::Color32::from_rgb(0, 128, 0)
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        if *st == LifeGameState::Dead {
            egui::Color32::from_rgb(0, 0, 0)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        }
    }

    fn update(&self, board: &mut Board<Self::CellState>) {
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
                    if self.alive.iter().map(|n| n+1).any(|n| n == nalive) {
                        LifeGameState::Alive
                    } else {
                        LifeGameState::Dead
                    }
                } else {
                    if self.birth.iter().any(|n| *n == nalive) {
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
