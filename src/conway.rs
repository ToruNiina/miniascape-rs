use crate::app::{Board, Rule, State};
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LifeGameState {
    Dead,
    Alive,
}

impl std::default::Default for LifeGameState {
    fn default() -> Self {
        LifeGameState::Dead
    }
}

impl State for LifeGameState {
    fn flip(&mut self) {
        if *self == LifeGameState::Dead {
            *self = LifeGameState::Alive;
        } else {
            *self = LifeGameState::Dead;
        }
    }

    fn color(&self) -> egui::Color32 {
        if *self == LifeGameState::Dead {
            egui::Color32::from_rgb(0, 0, 0)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        }
    }

    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        let distr = Bernoulli::new(0.3).expect("we know 0 < 0.3 < 1.");
        if distr.sample(rng) {
            *self = LifeGameState::Alive;
        } else {
            *self = LifeGameState::Dead;
        }
    }

    fn clear(&mut self) {
        *self = LifeGameState::Dead;
    }
}

pub struct LifeGameRule {}

impl Rule for LifeGameRule {
    type CellState = LifeGameState;

    fn background() -> egui::Color32 {
        egui::Color32::from_rgb(0, 128, 0)
    }

    fn update(board: &mut Board) {
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
                let board_is_alive = *board.cell_at(i, j) == LifeGameState::Alive;

                let buf = board.bufcell_at_mut(i, j);
                *buf = if nalive == 3 || (board_is_alive && nalive == 4) {
                    LifeGameState::Alive
                } else {
                    LifeGameState::Dead
                };
            }
        }
        std::mem::swap(&mut board.chunks, &mut board.buffer);
    }
}
