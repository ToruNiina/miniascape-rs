use crate::app::App;
use crate::board::{HexGrid, SquareGrid};
use crate::rule::{HexGridNeighborhood, MooreNeighborhood, VonNeumannNeighborhood};

use crate::gray_scott::{GrayScottRule, GrayScottState};
use crate::lifegame::{GeneralizedLifeGameRule, HighLifeRule, LifeGameRule, LifeGameState};
use crate::wireworld::{WireWorldRule, WireWorldState};

#[derive(Default)]
pub struct WrapApp {
    apps: Vec<(String, Box<dyn eframe::App>)>,
    focus: Option<usize>,

    life_game_rule: String,
}

impl WrapApp {
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

impl eframe::App for WrapApp {
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
                        Box::new(App::<
                            8,
                            MooreNeighborhood,
                            LifeGameRule,
                            SquareGrid<LifeGameState>,
                        >::default()),
                    ));
                }
                if ui.button("start highlife").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "HighLife".to_string(),
                        Box::new(App::<
                            8,
                            MooreNeighborhood,
                            HighLifeRule,
                            SquareGrid<LifeGameState>,
                        >::default()),
                    ));
                }
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if ui.button("start lifegame with specified rule").clicked() {
                        // convert `23/3` into [2, 3] and [3]
                        if GeneralizedLifeGameRule::is_valid_rule(&self.life_game_rule) {
                            self.focus = Some(self.apps.len());
                            self.apps.push((
                                self.life_game_rule.clone(),
                                Box::new(App::<
                                    8,
                                    MooreNeighborhood,
                                    GeneralizedLifeGameRule,
                                    SquareGrid<LifeGameState>,
                                >::new(
                                    GeneralizedLifeGameRule::from_rule(&self.life_game_rule),
                                )),
                            ));
                        }
                    }
                    ui.label("specify rule like: `23/3`");
                    let _ = ui.add(egui::TextEdit::singleline(&mut self.life_game_rule));
                });
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if ui.button("start hex lifegame with specified rule").clicked() {
                        // convert `23/3` into [2, 3] and [3]
                        if GeneralizedLifeGameRule::is_valid_rule(&self.life_game_rule) {
                            self.focus = Some(self.apps.len());
                            self.apps.push((
                                self.life_game_rule.clone(),
                                Box::new(App::<
                                    6,
                                    HexGridNeighborhood,
                                    GeneralizedLifeGameRule,
                                    HexGrid<LifeGameState>,
                                >::new(
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
                        Box::new(App::<
                            8,
                            MooreNeighborhood,
                            WireWorldRule,
                            SquareGrid<WireWorldState>,
                        >::default()),
                    ));
                }
                if ui.button("start Gray-Scott").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "Gray-Scott".to_string(),
                        Box::new(App::<
                            4,
                            VonNeumannNeighborhood,
                            GrayScottRule,
                            SquareGrid<GrayScottState>,
                        >::default()),
                    ));
                }
            });
        }
    }
}
