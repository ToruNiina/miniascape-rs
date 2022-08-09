use crate::rule::{Neighbors, Rule, State};
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
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(self, LifeGameState::Dead, "Dead");
        ui.radio_value(self, LifeGameState::Alive, "Alive");
    }
}

// ---------------------------------------------------------------------------

pub struct LifeGameRule {
    background: egui::Color32,
    alive_color: egui::Color32,
    dead_color: egui::Color32,
}

impl Default for LifeGameRule {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl<const N: usize, Neighborhood: Neighbors<N>> Rule<N, Neighborhood> for LifeGameRule {
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

    fn default_state(&self) -> Self::CellState {
        LifeGameState::Dead
    }

    fn randomize<R: Rng>(&self, rng: &mut R) -> Self::CellState {
        let distr = Bernoulli::new(0.3).expect("we know 0 < 0.3 < 1.");
        if distr.sample(rng) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn next(&self, st: Self::CellState) -> Self::CellState {
        if st == LifeGameState::Dead {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> Self::CellState {
        let n_alive: u32 = neighbor.map(|c| if c == LifeGameState::Alive { 1 } else { 0 }).sum();

        // 23/3
        if n_alive == 3 || (center == LifeGameState::Alive && n_alive == 2) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.background,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.alive_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.dead_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
    }
}

// ---------------------------------------------------------------------------

pub struct HighLifeRule {
    background: egui::Color32,
    alive_color: egui::Color32,
    dead_color: egui::Color32,
}

impl Default for HighLifeRule {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl<const N: usize, Neighborhood: Neighbors<N>> Rule<N, Neighborhood> for HighLifeRule {
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

    fn default_state(&self) -> Self::CellState {
        LifeGameState::Dead
    }

    fn randomize<R: Rng>(&self, rng: &mut R) -> Self::CellState {
        let distr = Bernoulli::new(0.3).expect("we know 0 < 0.3 < 1.");
        if distr.sample(rng) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn next(&self, st: Self::CellState) -> Self::CellState {
        if st == LifeGameState::Dead {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> Self::CellState {
        let center_is_alive = center == LifeGameState::Alive;
        let n_alive: u32 = neighbor.map(|c| if c == LifeGameState::Alive { 1 } else { 0 }).sum();

        // 23/36
        if n_alive == 3 || (center_is_alive && n_alive == 2) || (!center_is_alive && n_alive == 6) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.background,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.alive_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.dead_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
    }
}

// ----------------------------------------------------------------------------

pub struct GeneralizedLifeGameRule {
    alive: ArrayVec<u32, 9>, // number of neighboring cells is in [0, 8]
    birth: ArrayVec<u32, 9>,

    rule: String,
    show_err_msg_about_rule: bool,

    background: egui::Color32,
    alive_color: egui::Color32,
    dead_color: egui::Color32,
}

impl Default for GeneralizedLifeGameRule {
    fn default() -> Self {
        Self {
            alive: (&[2_u32, 3_u32] as &[_]).try_into().unwrap(),
            birth: (&[3_u32] as &[_]).try_into().unwrap(),
            rule: "23/3".to_string(),
            show_err_msg_about_rule: false,
            background: egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl GeneralizedLifeGameRule {
    pub fn new(alive: Vec<u32>, birth: Vec<u32>) -> Self {
        let rule = format!(
            "{}/{}",
            alive.iter().fold("".to_string(), |acc, x| acc + &x.to_string()),
            birth.iter().fold("".to_string(), |acc, x| acc + &x.to_string())
        );
        Self {
            alive: ArrayVec::from_iter(alive.into_iter()),
            birth: ArrayVec::from_iter(birth.into_iter()),
            rule,
            show_err_msg_about_rule: false,
            background: egui::Color32::from_rgb(0, 128, 0),
            alive_color: egui::Color32::from_rgb(0, 255, 0),
            dead_color: egui::Color32::from_rgb(0, 0, 0),
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

impl<const N: usize, Neighborhood: Neighbors<N>> Rule<N, Neighborhood> for GeneralizedLifeGameRule {
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

    fn default_state(&self) -> Self::CellState {
        LifeGameState::Dead
    }

    fn randomize<R: Rng>(&self, rng: &mut R) -> Self::CellState {
        let distr = Bernoulli::new(0.3).expect("we know 0 < 0.3 < 1.");
        if distr.sample(rng) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn next(&self, st: Self::CellState) -> Self::CellState {
        if st == LifeGameState::Dead {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> Self::CellState {
        let center_is_alive = center == LifeGameState::Alive;
        let n_alive: u32 = neighbor.map(|c| if c == LifeGameState::Alive { 1 } else { 0 }).sum();

        let meet_alive_rule = self.alive.iter().any(|n| *n == n_alive);
        let meet_birth_rule = self.birth.iter().any(|n| *n == n_alive);

        if (center_is_alive && meet_alive_rule) || (!center_is_alive && meet_birth_rule) {
            LifeGameState::Alive
        } else {
            LifeGameState::Dead
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("rule {survive}/{birth} (e.g. `23/3`)");
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
            ui,
            &mut self.background,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Live Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.alive_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Dead Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.dead_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
    }
}
