use crate::rule::{Neighbors, Rule, State};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, std::fmt::Debug)]
pub enum RockPaperScissorsState {
    Rock,
    Paper,
    Scissors,
}

impl std::default::Default for RockPaperScissorsState {
    fn default() -> Self {
        RockPaperScissorsState::Rock
    }
}

impl State for RockPaperScissorsState {
    fn inspect(&mut self, ui: &mut egui::Ui, _buf: &mut String) {
        ui.radio_value(self, RockPaperScissorsState::Rock, "Rock");
        ui.radio_value(self, RockPaperScissorsState::Paper, "Paper");
        ui.radio_value(self, RockPaperScissorsState::Scissors, "Scissors");
    }
}

pub struct RockPaperScissorsRule {
    background: egui::Color32,
    rock_color: egui::Color32,
    paper_color: egui::Color32,
    scissors_color: egui::Color32,
    threshold: u32,
}

impl Default for RockPaperScissorsRule {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(0, 0, 0),
            rock_color: egui::Color32::from_rgb(0, 0, 255),
            paper_color: egui::Color32::from_rgb(0, 255, 0),
            scissors_color: egui::Color32::from_rgb(255, 0, 0),
            threshold: 3,
        }
    }
}

impl<N: Neighbors> Rule<N> for RockPaperScissorsRule {
    type CellState = RockPaperScissorsState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> anyhow::Result<egui::Color32> {
        match *st {
            RockPaperScissorsState::Rock => Ok(self.rock_color),
            RockPaperScissorsState::Paper => Ok(self.paper_color),
            RockPaperScissorsState::Scissors => Ok(self.scissors_color),
        }
    }

    fn default_state(&self) -> anyhow::Result<Self::CellState> {
        Ok(RockPaperScissorsState::Rock)
    }

    fn randomize<R: Rng>(&self, rng: &mut R) -> anyhow::Result<Self::CellState> {
        match rng.gen_range(0..3) {
            0 => Ok(RockPaperScissorsState::Rock),
            1 => Ok(RockPaperScissorsState::Paper),
            2 => Ok(RockPaperScissorsState::Scissors),
            _ => unreachable!(),
        }
    }

    fn next(&self, st: Self::CellState) -> anyhow::Result<Self::CellState> {
        match st {
            RockPaperScissorsState::Rock => Ok(RockPaperScissorsState::Paper),
            RockPaperScissorsState::Paper => Ok(RockPaperScissorsState::Scissors),
            RockPaperScissorsState::Scissors => Ok(RockPaperScissorsState::Rock),
        }
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> anyhow::Result<Self::CellState> {
        let wins = match center {
            RockPaperScissorsState::Rock => RockPaperScissorsState::Paper,
            RockPaperScissorsState::Paper => RockPaperScissorsState::Scissors,
            RockPaperScissorsState::Scissors => RockPaperScissorsState::Rock,
        };

        let n_wins: u32 = neighbor.map(|c| if c == wins { 1 } else { 0 }).sum();

        Ok(if n_wins >= self.threshold { wins } else { center })
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        _on_side_panel: bool,
    ) -> anyhow::Result<()> {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.background,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Rock Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.rock_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Paper Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.paper_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );

        ui.label("Scissors Cell Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.scissors_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );

        ui.add(
            egui::Slider::new(&mut self.threshold, 0..=N::num_neighbors() as u32)
                .text("win/lose threshold"),
        );

        Ok(())
    }
}
