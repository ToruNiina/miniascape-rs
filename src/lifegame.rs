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

pub struct LifeGameRule {
    background:  egui::Color32,
    alive_color: egui::Color32,
    dead_color:  egui::Color32,
}

impl Default for LifeGameRule {
    fn default() -> Self {
        Self {
            background:  egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color:  egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl Rule for LifeGameRule {
    type CellState = LifeGameState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        if *st == LifeGameState::Dead {
            self.dead_color
        } else {
            self.alive_color
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

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.background, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.alive_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.dead_color, egui::widgets::color_picker::Alpha::Opaque);
    }
}

// ---------------------------------------------------------------------------

pub struct HighLifeRule {
    background:  egui::Color32,
    alive_color: egui::Color32,
    dead_color:  egui::Color32,
}

impl Default for HighLifeRule {
    fn default() -> Self {
        Self {
            background:  egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color:  egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl Rule for HighLifeRule {
    type CellState = LifeGameState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        if *st == LifeGameState::Dead {
            self.dead_color
        } else {
            self.alive_color
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

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.background, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.alive_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.dead_color, egui::widgets::color_picker::Alpha::Opaque);
    }
}

// ----------------------------------------------------------------------------

pub struct GeneralizedLifeGameRule {
    alive: ArrayVec<u32, 9>, // number of neighboring cells is in [0, 8]
    birth: ArrayVec<u32, 9>,

    rule: String,
    show_err_msg_about_rule: bool,

    background:  egui::Color32,
    alive_color: egui::Color32,
    dead_color:  egui::Color32,
}

impl Default for GeneralizedLifeGameRule {
    fn default() -> Self {
        Self {
            alive: (&[2_u32, 3_u32] as &[_]).try_into().unwrap(),
            birth: (&[3_u32] as &[_]).try_into().unwrap(),
            rule: "23/3".to_string(),
            show_err_msg_about_rule: false,
            background:  egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color:  egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl GeneralizedLifeGameRule {
    pub fn new(alive: Vec<u32>, birth: Vec<u32>) -> Self {
        let rule = format!("{}/{}",
            alive.iter().fold("".to_string(), |acc, x| acc + &x.to_string()),
            birth.iter().fold("".to_string(), |acc, x| acc + &x.to_string()));
        Self {
            alive: ArrayVec::from_iter(alive.into_iter()),
            birth: ArrayVec::from_iter(birth.into_iter()),
            rule: rule,
            show_err_msg_about_rule: false,
            background:  egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color:  egui::Color32::from_rgb(0, 0, 0),
        }
    }

    pub fn is_valid_rule(rule: &str) -> bool {
        let has_separator = rule.chars().filter(|c| *c == '/').count() == 1;
        let has_digit_only = rule.chars().all(|c| c.is_ascii_digit() || c == '/');
        has_separator && has_digit_only
    }

    pub fn parse_rule(rule: &str) -> Option<(Vec<u32>, Vec<u32>)> {
        if !Self::is_valid_rule(rule) {
            return None;
        }

        // convert `23/3` into [2, 3] and [3]
        let alive_birth: Vec<&str> = rule.split('/').collect();

        let alive = alive_birth[0].chars().map(|c| c.to_digit(10).unwrap()).collect();
        let birth = alive_birth[1].chars().map(|c| c.to_digit(10).unwrap()).collect();

        Some((alive, birth))
    }

    pub fn from_rule(rule: &str) -> Self {
        assert!(Self::is_valid_rule(rule));

        let (alive, birth) = Self::parse_rule(rule).unwrap();

        GeneralizedLifeGameRule::new(alive, birth)
    }
}

impl Rule for GeneralizedLifeGameRule {
    type CellState = LifeGameState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        if *st == LifeGameState::Dead {
            self.dead_color
        } else {
            self.alive_color
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

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("specify rule like: `23/3`");
        ui.horizontal_wrapped(|ui| {
            if ui.add(egui::TextEdit::singleline(&mut self.rule)).changed() {
                self.show_err_msg_about_rule = false;
            }

            if ui.button("Apply").clicked() {
                if let Some((alive, birth)) = Self::parse_rule(&self.rule) {
                    self.alive = ArrayVec::from_iter(alive.into_iter());
                    self.birth = ArrayVec::from_iter(birth.into_iter());
                } else {
                    self.show_err_msg_about_rule = true;
                }
            }
        });
        if self.show_err_msg_about_rule {
            ui.label("Invalid Rule");
        }
        ui.separator();

        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.background, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.alive_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.dead_color, egui::widgets::color_picker::Alpha::Opaque);
    }
}
