use crate::rule::{MooreNeighborhood, Rule, State};
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
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(self, WireWorldState::Void, "Void");
        ui.radio_value(self, WireWorldState::Head, "Head");
        ui.radio_value(self, WireWorldState::Tail, "Tail");
        ui.radio_value(self, WireWorldState::Wire, "Wire");
    }
}

pub struct WireWorldRule {
    grid_color: egui::Color32,
    void_color: egui::Color32,
    wire_color: egui::Color32,
    head_color: egui::Color32,
    tail_color: egui::Color32,
}

impl Default for WireWorldRule {
    fn default() -> Self {
        Self {
            grid_color: egui::Color32::from_rgb(128, 128, 0),
            void_color: egui::Color32::from_rgb(0, 0, 0),
            wire_color: egui::Color32::from_rgb(255, 255, 0),
            head_color: egui::Color32::from_rgb(0, 0, 255),
            tail_color: egui::Color32::from_rgb(255, 0, 0),
        }
    }
}

impl Rule<8, MooreNeighborhood> for WireWorldRule {
    type CellState = WireWorldState;

    fn background(&self) -> egui::Color32 {
        self.grid_color
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        match *st {
            WireWorldState::Void => self.void_color,
            WireWorldState::Head => self.head_color,
            WireWorldState::Tail => self.tail_color,
            WireWorldState::Wire => self.wire_color,
        }
    }

    fn default_state(&self) -> Self::CellState {
        WireWorldState::Void
    }

    fn randomize<R: Rng>(&self, rng: &mut R) -> Self::CellState {
        match rng.gen_range(0..4) {
            0 => WireWorldState::Void,
            1 => WireWorldState::Head,
            2 => WireWorldState::Tail,
            3 => WireWorldState::Wire,
            _ => unreachable!(),
        }
    }

    fn next(&self, st: Self::CellState) -> Self::CellState {
        match st {
            WireWorldState::Void => WireWorldState::Wire,
            WireWorldState::Wire => WireWorldState::Head,
            WireWorldState::Head => WireWorldState::Tail,
            WireWorldState::Tail => WireWorldState::Void,
        }
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> Self::CellState {
        match center {
            WireWorldState::Void => WireWorldState::Void,
            WireWorldState::Head => WireWorldState::Tail,
            WireWorldState::Tail => WireWorldState::Wire,
            WireWorldState::Wire => {
                let nheads: u32 =
                    neighbor.map(|c| if c == WireWorldState::Head { 1 } else { 0 }).sum();
                if nheads == 1 || nheads == 2 {
                    WireWorldState::Head
                } else {
                    WireWorldState::Wire
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.grid_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Void Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.void_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Wire Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.wire_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Electron Head Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.head_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
        ui.separator();

        ui.label("Electron Tail Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.tail_color,
            egui::widgets::color_picker::Alpha::Opaque,
        );
    }
}
