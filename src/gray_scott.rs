use crate::board::Board;
use crate::rule::{Rule, State};
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct GrayScottState {
    u: f32,
    v: f32,
}

impl std::default::Default for GrayScottState {
    fn default() -> Self {
        Self{
            u: 0.0,
            v: 0.0,
        }
    }
}

impl State for GrayScottState {
    fn next(&self) -> Self {
        Self{
            u: (self.u + 0.01).min(1.0),
            v: self.v,
        }
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
    dt:  f32,
    dx:  f32,
    d_u: f32, // D_u
    d_v: f32, // D_v
    f:   f32,
    k:   f32,
    n:   u32,
}

impl std::default::Default for GrayScottRule {
    fn default() -> Self {
        Self {
            dt:   0.1,
            dx:   0.1,
            d_u:  0.001, // D_u
            d_v:  0.005, // D_v
            f:    0.09,
            k:    0.06,
            n:    40,
        }
    }
}

impl Rule for GrayScottRule {
    type CellState = GrayScottState;

    fn background(&self) -> egui::Color32 {
        egui::Color32::from_rgb(0, 0, 0)
    }

    fn color(&self, st: &Self::CellState) -> egui::Color32 {
        let r = 16_u8;
        let g = (st.v * 256.0).clamp(0.0, 255.0) as u8;
        let b = (st.u * 256.0).clamp(0.0, 255.0) as u8;
        egui::Color32::from_rgb(r, g, b)
    }

    fn update(&self, board: &mut Board<Self::CellState>) {
        for _ in 0..self.n {
            let Self { dt, dx, d_u, d_v, f, k, n:_ } = *self;
            let rdx2 = 1.0 / (dx * dx);

            for j in 0..board.height() {
                let yprev = if j == 0 { board.height() - 1 } else { j - 1 };
                let ynext = if j == board.height() - 1 { 0 } else { j + 1 };
                for i in 0..board.width() {
                    let xprev = if i == 0 { board.width() - 1 } else { i - 1 };
                    let xnext = if i == board.width() - 1 { 0 } else { i + 1 };

                    let u0 = board.cell_at(i, j).u;
                    let v0 = board.cell_at(i, j).v;

                    let mut lu = -4.0 * u0;
                    let mut lv = -4.0 * v0;
                    for (nx, ny) in [(i, yprev), (i, ynext), (xprev, j), (xnext, j)] {
                        let GrayScottState{u, v} = *board.cell_at(nx, ny);
                        lu += u;
                        lv += v;
                    }
                    lu *= rdx2;
                    lv *= rdx2;

                    // du/dt = Du * nabla^2 u + u^2*v - (f + k) u
                    // dv/dt = Dv * nabla^2 v - u^2*v + f(1 - v)
                    let u = u0 + dt * (d_u * lu + u0 * u0 * v0 - (f + k) * u0);
                    let v = v0 + dt * (d_v * lv - u0 * u0 * v0 + (1.0 - v0) * f);
                    *board.bufcell_at_mut(i, j) = GrayScottState{u, v};
                }
            }
            std::mem::swap(&mut board.chunks, &mut board.buffer);
        }
    }
    // draw rule-specific part of UI
}
