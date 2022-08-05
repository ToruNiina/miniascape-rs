use crate::rule::{Rule, State, VonNeumannNeighborhood};
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct GrayScottState {
    u: f32,
    v: f32,
}

impl std::default::Default for GrayScottState {
    fn default() -> Self {
        Self { u: 0.0, v: 0.0 }
    }
}

impl State for GrayScottState {
    fn next(&self) -> Self {
        Self { u: (self.u + 0.01).min(1.0), v: self.v }
    }

    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        let distr = Uniform::new_inclusive(0.0, 1.0);
        self.u = distr.sample(rng);
        self.v = distr.sample(rng);
    }

    fn clear(&mut self) {
        self.u = 0.0;
        self.v = 0.0;
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.u, 0.0..=1.0).text("u"));
        ui.add(egui::Slider::new(&mut self.v, 0.0..=1.0).text("v"));
    }
}

///
/// du/dt = Du * nabla^2 u + u^2*v - (f + k) u
/// dv/dt = Dv * nabla^2 v - u^2*v + f(1 - v)
///
pub struct GrayScottRule {
    dt: f32,
    dx: f32,
    invdx2: f32,
    d_u: f32, // D_u
    d_v: f32, // D_v
    f: f32,
    k: f32,
    n: u32,

    u_color: egui::Color32,
    v_color: egui::Color32,
    background: egui::Color32,
}

impl std::default::Default for GrayScottRule {
    fn default() -> Self {
        Self {
            dt: 0.1,
            dx: 0.1,
            invdx2: 100.0, // 1/dx^2
            d_u: 0.001, // D_u
            d_v: 0.005, // D_v
            f: 0.09,
            k: 0.06,
            n: 40,
            u_color: egui::Color32::from_rgb(16, 0, 255),
            v_color: egui::Color32::from_rgb(16, 255, 0),
            background:  egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl Rule<4, VonNeumannNeighborhood> for GrayScottRule {
    type CellState = GrayScottState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        let (u_r, u_g, u_b) = (self.u_color.r(), self.u_color.g(), self.u_color.b());
        let (v_r, v_g, v_b) = (self.v_color.r(), self.v_color.g(), self.v_color.b());

        let r = (st.u * u_r as f32 + st.v * v_r as f32).clamp(0.0, 255.0) as u8;
        let g = (st.u * u_g as f32 + st.v * v_g as f32).clamp(0.0, 255.0) as u8;
        let b = (st.u * u_b as f32 + st.v * v_b as f32).clamp(0.0, 255.0) as u8;

        egui::Color32::from_rgb(r, g, b)
    }

    fn update(&self, center: Self::CellState, neighbor: impl Iterator<Item = Self::CellState>) -> Self::CellState {
        // TODO require von neumann neighborhood
        // currently it assumes that neighbor is in the following order:
        // (x, y) = [
        //   (-, -), (0, -), (+, -)
        //   (-, 0),         (+, 0)
        //   (-, +), (0, +), (+, +)
        // ]
        // 1, 3, 4, 6

        let u0 = center.u;
        let v0 = center.v;
        let (lu, lv) = neighbor.enumerate().filter_map(|(i, c)| {
            if i == 1 || i == 3 || i == 4 || i == 6 { Some(c) } else { None }
        }).fold((-4.0 * u0, -4.0 * v0), |acc, c| (acc.0 + c.u, acc.1 + c.v));

        let Self{dt, invdx2, d_u, d_v, f, k, ..} = *self;

        let u = u0 + dt * (d_u * lu * invdx2 + u0 * u0 * v0 - (f + k) * u0);
        let v = v0 + dt * (d_v * lv * invdx2 - u0 * u0 * v0 + (1.0 - v0) * f);

        Self::CellState{ u, v }
    }

    fn iteration_per_step(&self) -> u32 {
        self.n
    }

    fn ui(&mut self, ui: &mut egui::Ui) {

        ui.label(format!("dt = {}", self.dt));
        ui.label(format!("dx = {}", self.dx));

        ui.add(egui::Slider::new(&mut self.d_u, 0.0..=0.01).text("Du"));
        ui.add(egui::Slider::new(&mut self.d_v, 0.0..=0.01).text("Dv"));

        ui.add(egui::Slider::new(&mut self.f, 0.0..=0.1).text("f"));
        ui.add(egui::Slider::new(&mut self.k, 0.0..=0.1).text("k"));

        ui.add(egui::Slider::new(&mut self.n, 0..=100).text("how many time integrations per frame"));

        ui.separator();

        ui.label("Grid Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.background, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("u Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.u_color, egui::widgets::color_picker::Alpha::Opaque);
        ui.separator();

        ui.label("v Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui, &mut self.v_color, egui::widgets::color_picker::Alpha::Opaque);
    }
}
