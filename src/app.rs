use crate::board::Board;
use crate::rule::{Neighbors, Rule, State};

use rand::SeedableRng;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct App<const N: usize, Ne, R, B>
where
    Ne: Neighbors<N>,
    R: Rule<N, Ne>,
    B: Board<N, Ne, R>,
{
    rule: R,
    board: B,
    fix_board_size: bool,
    running: bool,
    inspector: Option<(usize, usize)>,
    inspector_indicator: bool,
    grid_width: f32,
    origin: egui::Pos2,
    grabbed: bool,
    cell_modifying: Option<R::CellState>,
    rng: rand::rngs::StdRng,
}

impl<const N: usize, Ne, R, B> Default for App<N, Ne, R, B>
where
    Ne: Neighbors<N>,
    R: Rule<N, Ne>,
    B: Board<N, Ne, R>,
{
    fn default() -> Self {
        Self {
            rule: Default::default(),
            board: B::new(8, 8),
            fix_board_size: false,
            running: false,
            inspector: None,
            inspector_indicator: true,
            grid_width: 32.0,
            origin: egui::Pos2::new(0.0, 0.0),
            grabbed: false,
            cell_modifying: None,
            rng: rand::rngs::StdRng::seed_from_u64(123456789),
        }
    }
}

/// to avoid context lock by ctx.input()
pub enum Clicked {
    Primary(usize, usize),
    Secondary(usize, usize),
    NotClicked,
}

impl<const N: usize, Ne, R, B> App<N, Ne, R, B>
where
    Ne: Neighbors<N>,
    R: Rule<N, Ne>,
    B: Board<N, Ne, R>,
{
    pub fn new(rule: R) -> Self {
        Self {
            rule,
            board: Board::new(8, 8),
            fix_board_size: false,
            running: false,
            inspector: None,
            inspector_indicator: true,
            grid_width: 32.0,
            origin: egui::Pos2::new(0.0, 0.0),
            grabbed: false,
            cell_modifying: None,
            rng: rand::rngs::StdRng::seed_from_u64(123456789),
        }
    }
    pub fn min_gridsize() -> f32 {
        8.0
    }
    pub fn max_gridsize() -> f32 {
        128.0
    }
    pub fn scroll_factor() -> f32 {
        1.0 / 128.0
    }

    pub fn clicked(&self, ctx: &egui::Context, region_min: egui::Pos2) -> Clicked {
        let pointer = &ctx.input().pointer;
        if !pointer.primary_down() && !pointer.secondary_down() {
            return Clicked::NotClicked;
        }

        let pos = pointer
            .interact_pos()
            .unwrap_or(egui::Pos2::new(-f32::INFINITY, -f32::INFINITY));

        let dx = pos.x - region_min.x + self.origin.x;
        let dy = pos.y - region_min.y + self.origin.y;

        if let Some((ix, iy)) = self.board.clicked(dx, dy, self.grid_width) {
            if pointer.primary_down() {
                Clicked::Primary(ix, iy)
            } else if pointer.secondary_down() {
                Clicked::Secondary(ix, iy)
            } else {
                Clicked::NotClicked
            }
        } else {
            Clicked::NotClicked
        }
    }
}

impl<const N: usize, Ne, R, B> eframe::App for App<N, Ne, R, B>
where
    Ne: Neighbors<N>,
    R: Rule<N, Ne>,
    B: Board<N, Ne, R>,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //         eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    #[allow(clippy::never_loop)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            self.board.update(&self.rule);
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            egui_extras::TableBuilder::new(ui)
                .column(egui_extras::Size::initial(100.0))
                .column(egui_extras::Size::remainder())
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("operation");
                    });
                    header.col(|ui| {
                        ui.heading("effect");
                    });
                })
                .body(|mut body| {
                    body.row(32.0, |mut row| {
                        row.col(|ui| {
                            ui.label("left click & drag");
                        });
                        row.col(|ui| {
                            ui.label("change state of a cell clicked");
                        });
                    });
                    body.row(32.0, |mut row| {
                        row.col(|ui| {
                            ui.label("wheel click & drag");
                        });
                        row.col(|ui| {
                            ui.label("grab the board and move it");
                        });
                    });
                    body.row(32.0, |mut row| {
                        row.col(|ui| {
                            ui.label("right click");
                        });
                        row.col(|ui| {
                            ui.label("modify cell state");
                        });
                    });
                });

            ui.separator(); // -------------------------------------------------

            ui.horizontal_wrapped(|ui| {
                ui.toggle_value(&mut self.running, "Run");

                if ui.button("Step").clicked() {
                    self.board.update(&self.rule);
                    ui.ctx().request_repaint();
                }
                if ui.button("Reset").clicked() {
                    self.board.clear(&self.rule);
                }
                if ui.button("Randomize").clicked() {
                    self.board.randomize(&self.rule, &mut self.rng);
                }
            });

            ui.separator(); // -------------------------------------------------

            let min_grid = Self::min_gridsize();
            let max_grid = Self::max_gridsize();
            ui.add(egui::Slider::new(&mut self.grid_width, min_grid..=max_grid).text("grid_width"));
            ui.checkbox(&mut self.fix_board_size, "Fix Board Size");

            ui.separator();
            ui.label("status:");
            ui.label(format!("current cells: {}x{}", self.board.width(), self.board.height()));
            ui.label(format!(
                "current chunks: {}x{}",
                self.board.n_chunks_x(),
                self.board.n_chunks_y()
            ));
            ui.label(format!("current origin: ({},{})", self.origin.x, self.origin.y));

            ui.separator(); // -------------------------------------------------

            self.rule.ui(ui);
        });

        {
            let pointer = &ctx.input().pointer;
            if self.grabbed {
                self.origin -= pointer.delta();
            }
            if pointer.middle_down() {
                self.grabbed = true;
            } else {
                self.grabbed = false;
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.running {
                ui.ctx().request_repaint();
            }

            // ----------------------------------------------------------------
            // First make a painter only for inside the region.
            let painter = egui::Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );

            let region = painter.clip_rect();

            // determine the number of chunks after zoom in/out
            let delta = self.grid_width;
            let regsize = region.max - region.min;

            // ----------------------------------------------------------------
            // zoom in/out by scroll
            {
                let scroll = ctx.input().scroll_delta.y * Self::scroll_factor();
                if scroll != 0.0 {
                    let new_grid_width = (self.grid_width * 1.1_f32.powf(scroll))
                        .clamp(Self::min_gridsize(), Self::max_gridsize())
                        .ceil();

                    let magnification = new_grid_width / self.grid_width;
                    let center = self.origin.to_vec2() + (regsize * 0.5);

                    self.origin = (center * magnification - regsize * 0.5).to_pos2();
                    self.grid_width = new_grid_width;
                }
            }

            // ----------------------------------------------------------------
            // expand board size if needed

            if !self.fix_board_size {
                let chunk_pxls = self.board.chunk_len() as f32 * delta;

                if self.origin.x < 0.0 {
                    let d = (self.origin.x / chunk_pxls).floor();
                    self.board.expand_x(d as isize);
                    self.origin.x -= chunk_pxls * d;
                    assert!(0.0 <= self.origin.x);
                }

                if self.board.width() as f32 * delta <= self.origin.x + regsize.x {
                    let dx = self.origin.x + regsize.x - self.board.width() as f32 * delta;
                    assert!(0.0 <= dx);
                    let d = (dx / chunk_pxls).ceil();
                    self.board.expand_x(d as isize);
                }

                if self.origin.y < 0.0 {
                    let d = (self.origin.y / chunk_pxls).floor();
                    self.board.expand_y(d as isize);
                    self.origin.y -= chunk_pxls * d;
                    assert!(0.0 <= self.origin.y);
                }
                if self.board.height() as f32 * delta <= self.origin.y + regsize.y {
                    let dy = self.origin.y + regsize.y - self.board.height() as f32 * delta;
                    assert!(0.0 <= dy);
                    let d = (dy / chunk_pxls).ceil();
                    self.board.expand_y(d as isize);
                }
            }

            // ----------------------------------------------------------------
            // draw board to the central panel

            self.board.paint(&painter, self.origin, delta, &self.rule);

            // ----------------------------------------------------------------
            // handle left/right click

            let clicked = self.clicked(ctx, region.min);

            // stop running and inspect cell state by right click
            if let Clicked::Secondary(ix, iy) = clicked {
                self.running = false;
                self.inspector = Some((ix, iy));
                self.cell_modifying = None;
            }
            if let Clicked::NotClicked = clicked {
                self.cell_modifying = None;
            }

            if let Some((ix, iy)) = self.inspector {
                let mut open = true;
                egui::Window::new("Cell Inspector").open(&mut open).show(ctx, |ui| {
                    ui.checkbox(&mut self.inspector_indicator, "Indicator");
                    self.board.cell_at_mut(ix, iy).inspect(ui);
                });
                if !open {
                    self.inspector = None;
                }

                // point the cell that is inspected by a line if inspector is opened

                if self.inspector_indicator {
                    let c = self.board.location(ix, iy, self.origin, region.min, delta);
                    let r = delta * 0.5_f32.sqrt();
                    painter.add(epaint::CircleShape::stroke(
                        c,
                        r,
                        epaint::Stroke {
                            width: 5.0,
                            color: egui::Color32::from_rgb(255, 255, 255),
                        },
                    ));
                    painter.add(epaint::CircleShape::stroke(
                        c,
                        r,
                        epaint::Stroke { width: 2.0, color: egui::Color32::from_rgb(0, 0, 0) },
                    ));
                }
            } else {
                // if inspector is closed, then we can click a cell
                if let Clicked::Primary(ix, iy) = clicked {
                    if let Some(next) = &self.cell_modifying {
                        *self.board.cell_at_mut(ix, iy) = *next;
                    } else {
                        let next = self.board.cell_at(ix, iy).next();
                        *self.board.cell_at_mut(ix, iy) = next;
                        self.cell_modifying = Some(next);
                    }
                }
            }

            // detect debug build
            egui::warn_if_debug_build(ui);
        });
    }
}
