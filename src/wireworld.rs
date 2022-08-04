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

impl Rule for WireWorldRule {
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

    fn ui(&mut self, ui: &mut egui::Ui) {

        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.grid_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Void Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.void_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Wire Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.wire_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Electron Head Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.head_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("Electron Tail Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.tail_color, egui::widgets::color_picker::Alpha::Opaque);
    }
}
