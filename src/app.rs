use crate::board::Board;
use crate::rule::{Rule, State};

use crate::gray_scott::GrayScottRule;
use crate::lifegame::{GeneralizedLifeGameRule, HighLifeRule, LifeGameRule};
use crate::wireworld::WireWorldRule;

use rand::SeedableRng;
use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
//   ___                  _      _
//  / __|___ _ _  ___ _ _(_)__  /_\  _ __ _ __
// | (_ / -_) ' \/ -_) '_| / _|/ _ \| '_ \ '_ \
//  \___\___|_||_\___|_| |_\__/_/ \_\ .__/ .__/
//                                  |_|  |_|

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GenericApp<R: Rule> {
    #[serde(skip)]
    rule: R,
    #[serde(skip)]
    board: Board<R::CellState>,
    fix_board_size: bool,
    #[serde(skip)]
    running: bool,
    #[serde(skip)]
    inspector: Option<(usize, usize)>,
    inspector_indicator: bool,
    #[serde(skip)]
    grid_width: f32,
    #[serde(skip)]
    origin: egui::Pos2,
    #[serde(skip)]
    grabbed: bool,
    #[serde(skip)]
    cell_modifying: Option<R::CellState>,
    #[serde(skip)]
    rng: rand::rngs::StdRng,
}

impl<R: Rule> Default for GenericApp<R> {
    fn default() -> Self {
        Self {
            rule: Default::default(),
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
}

impl<R: Rule> GenericApp<R> {
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
}

/// to avoid context lock by ctx.input()
pub enum Clicked {
    Primary(usize, usize),
    Secondary(usize, usize),
    NotClicked,
}

impl<R: Rule> GenericApp<R> {
    fn clicked(&self, ctx: &egui::Context, region_min: egui::Pos2) -> Clicked {
        let pointer = &ctx.input().pointer;
        if !pointer.primary_down() && !pointer.secondary_down() {
            return Clicked::NotClicked;
        }

        let pos = pointer
            .interact_pos()
            .unwrap_or(egui::Pos2::new(-f32::INFINITY, -f32::INFINITY));

        let dxy = pos - region_min;
        if dxy.x < 0.0 || dxy.y < 0.0 {
            return Clicked::NotClicked;
        }

        let rdelta = 1.0 / self.grid_width;
        let ix = ((dxy.x + self.origin.x) * rdelta).floor() as usize;
        let iy = ((dxy.y + self.origin.y) * rdelta).floor() as usize;
        if self.board.width() <= ix || self.board.height() <= iy {
            return Clicked::NotClicked;
        }

        if pointer.primary_down() {
            Clicked::Primary(ix, iy)
        } else if pointer.secondary_down() {
            Clicked::Secondary(ix, iy)
        } else {
            Clicked::NotClicked
        }
    }
}

impl<R: Rule> eframe::App for GenericApp<R> {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
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
                    self.board.clear();
                }
                if ui.button("Randomize").clicked() {
                    self.board.randomize(&mut self.rng);
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
                let chunk_pxls = Board::<R::CellState>::chunk_len() as f32 * delta;

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

            self.board.paint(&painter, self.origin, delta, self.rule.background(), |s| {
                self.rule.color(s)
            });

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
                    let cx = (ix as f32 + 0.5) * delta - self.origin.x + region.min.x;
                    let cy = (iy as f32 + 0.5) * delta - self.origin.y + region.min.y;
                    let r  = delta * 0.5_f32.sqrt();
                    painter.add(epaint::CircleShape::stroke(
                            egui::Pos2{x: cx, y: cy},
                            r,
                            epaint::Stroke{
                                width: 5.0,
                                color: egui::Color32::from_rgb(255, 255, 255)
                            },
                        ));
                    painter.add(epaint::CircleShape::stroke(
                            egui::Pos2{x: cx, y: cy},
                            r,
                            epaint::Stroke{
                                width: 2.0,
                                color: egui::Color32::from_rgb(0, 0, 0)
                            },
                        ));
                }
            } else {
                // if inspector is closed, then we can click a cell
                if let Clicked::Primary(ix, iy) = clicked {
                    if let Some(next) = &self.cell_modifying {
                        *self.board.cell_at_mut(ix, iy) = next.clone();
                    } else {
                        let next = self.board.cell_at(ix, iy).next();
                        *self.board.cell_at_mut(ix, iy) = next.clone();
                        self.cell_modifying = Some(next);
                    }
                }
            }

            // detect debug build
            egui::warn_if_debug_build(ui);
        });
    }
}

// ----------------------------------------------------------------------------
//    _
//   /_\  _ __ _ __
//  / _ \| '_ \ '_ \
// /_/ \_\ .__/ .__/
//       |_|  |_|

#[derive(Default)]
pub struct App {
    apps: Vec<(String, Box<dyn eframe::App>)>,
    focus: Option<usize>,

    life_game_rule: String,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                // default page
                if ui.selectable_label(self.focus == None, "Home").clicked() {
                    self.focus = None;
                };

                // list of running apps
                let mut remove = None;
                for (idx, (name, _)) in self.apps.iter().enumerate() {
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::style::Margin {
                            top: 0.0_f32,
                            bottom: 0.0_f32,
                            left: 0.0_f32,
                            right: 0.0_f32,
                        })
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui.selectable_label(self.focus == Some(idx), name).clicked() {
                                    self.focus = Some(idx);
                                }
                                if ui.add(egui::Button::new("🗙").frame(false)).clicked() {
                                    remove = Some(idx);
                                }
                            });
                        });
                }
                if let Some(idx) = remove {
                    self.apps.remove(idx);
                    if self.focus == Some(idx) {
                        if self.apps.is_empty() {
                            self.focus = None;
                        } else if self.apps.len() <= idx {
                            self.focus = Some(self.apps.len() - 1);
                        } else {
                            self.focus = Some(idx);
                        }
                    } else if let Some(cur_focus) = self.focus {
                        if idx < cur_focus {
                            self.focus = Some(cur_focus - 1)
                        }
                    }
                    // if none, then still none.
                }
            });
        });

        egui::TopBottomPanel::bottom("acknowledge").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        // run only one app at a time
        if let Some(idx) = self.focus {
            assert!(idx <= self.apps.len());
            self.apps[idx].1.update(ctx, frame);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                if ui.button("start life game").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "LifeGame".to_string(),
                        Box::new(GenericApp::<LifeGameRule>::default()),
                    ));
                }
                if ui.button("start highlife").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "HighLife".to_string(),
                        Box::new(GenericApp::<HighLifeRule>::default()),
                    ));
                }
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if ui.button("start lifegame with specified rule").clicked() {
                        // convert `23/3` into [2, 3] and [3]
                        if GeneralizedLifeGameRule::is_valid_rule(&self.life_game_rule) {
                            self.focus = Some(self.apps.len());
                            self.apps.push((
                                self.life_game_rule.clone(),
                                Box::new(GenericApp::<GeneralizedLifeGameRule>::new(
                                    GeneralizedLifeGameRule::from_rule(&self.life_game_rule),
                                )),
                            ));
                        }
                    }
                    ui.label("specify rule like: `23/3`");
                    let _ = ui.add(egui::TextEdit::singleline(&mut self.life_game_rule));
                });

                if ui.button("start WireWorld").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "WireWorld".to_string(),
                        Box::new(GenericApp::<WireWorldRule>::default()),
                    ));
                }
                if ui.button("start Gray-Scott").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "Gray-Scott".to_string(),
                        Box::new(GenericApp::<GrayScottRule>::default()),
                    ));
                }
            });
        }
    }
}
