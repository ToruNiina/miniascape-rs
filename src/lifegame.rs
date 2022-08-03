use crate::board::Board;
use crate::rule::{Rule, State};
use arrayvec::ArrayVec;
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
    fn next(&self) -> Self {
        if *self == LifeGameState::Dead {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
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

    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(self, LifeGameState::Dead, "Dead");
        ui.radio_value(self, LifeGameState::Alive, "Alive");
    }
}

// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LifeGameRule {}

impl Rule for LifeGameRule {
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
    // draw rule-specific part of UI
}

// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct HighLifeRule {}

impl Rule for HighLifeRule {
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
                *buf = if nalive == 3
                    || (self_is_alive && nalive == 4)
                    || (!self_is_alive && nalive == 6)
                {
                    LifeGameState::Alive
                } else {
                    LifeGameState::Dead
                }
            }
        }
        std::mem::swap(&mut board.chunks, &mut board.buffer);
    }
    // draw rule-specific part of UI
}

// ----------------------------------------------------------------------------

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

    pub fn is_correct_rule(rule: &str) -> bool {
        let has_separator = rule.chars().filter(|c| *c == '/').count() == 1;
        let has_digit_only = rule.chars().all(|c| c.is_ascii_digit() || c == '/');
        has_separator && has_digit_only
    }

    pub fn parse_rule(rule: &str) -> Self {
        assert!(Self::is_correct_rule(rule));

        // convert `23/3` into [2, 3] and [3]
        let alive_birth: Vec<&str> = rule.split('/').collect();

        let alive = alive_birth[0].chars().map(|c| c.to_digit(10).unwrap()).collect();
        let birth = alive_birth[1].chars().map(|c| c.to_digit(10).unwrap()).collect();

        GeneralizedLifeGameRule::new(alive, birth)
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
                *buf = if (self_is_alive && self.alive.iter().map(|n| n + 1).any(|n| n == nalive))
                    || (!self_is_alive && self.birth.iter().any(|n| *n == nalive))
                {
                    LifeGameState::Alive
                } else {
                    LifeGameState::Dead
                };
            }
        }
        std::mem::swap(&mut board.chunks, &mut board.buffer);
    }
    // draw rule-specific part of UI
}
