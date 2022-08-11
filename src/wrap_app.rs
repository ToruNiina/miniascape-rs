use crate::app::App;
use crate::board::{HexGrid, SquareGrid};
use crate::rule::{HexGridNeighborhood, MooreNeighborhood, VonNeumannNeighborhood};

use crate::dynamic_rule::{DynamicRule, DynamicState};
use crate::gray_scott::{GrayScottRule, GrayScottState};
use crate::lifegame::{GeneralizedLifeGameRule, HighLifeRule, LifeGameRule, LifeGameState};
use crate::wireworld::{WireWorldRule, WireWorldState};

use egui_extras::RetainedImage;

pub struct WrapApp {
    apps: Vec<(String, Box<dyn eframe::App>)>,
    focus: Option<usize>,

    life_game_rule: String,

    thumbnail_lifegame: RetainedImage,
    thumbnail_generalized_lifegame: RetainedImage,
    thumbnail_hexlife: RetainedImage,
    thumbnail_highlife: RetainedImage,
    thumbnail_wireworld: RetainedImage,
    thumbnail_gray_scott: RetainedImage,
    card_height: f32,
    card_width: f32,
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

        Self {
            apps: Vec::new(),
            focus: None,
            life_game_rule: "23/3".to_string(),
            thumbnail_lifegame: RetainedImage::from_image_bytes(
                "thumbnail_lifegame.png",
                include_bytes!("images/thumbnail_lifegame.png"),
            )
            .unwrap(),
            thumbnail_generalized_lifegame: RetainedImage::from_image_bytes(
                "thumbnail_generalized_lifegame.png",
                include_bytes!("images/thumbnail_generalized_lifegame.png"),
            )
            .unwrap(),
            thumbnail_hexlife: RetainedImage::from_image_bytes(
                "thumbnail_hexlife.png",
                include_bytes!("images/thumbnail_hexlife.png"),
            )
            .unwrap(),
            thumbnail_highlife: RetainedImage::from_image_bytes(
                "thumbnail_highlife.png",
                include_bytes!("images/thumbnail_highlife.png"),
            )
            .unwrap(),
            thumbnail_wireworld: RetainedImage::from_image_bytes(
                "thumbnail_wireworld.png",
                include_bytes!("images/thumbnail_wireworld.png"),
            )
            .unwrap(),
            thumbnail_gray_scott: RetainedImage::from_image_bytes(
                "thumbnail_gray_scott.png",
                include_bytes!("images/thumbnail_gray_scott.png"),
            )
            .unwrap(),
            card_height: 260.0,
            card_width: 320.0,
        }
    }

    fn draw_lifegame_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_lifegame.texture_id(ctx),
                        self.thumbnail_lifegame.size_vec2(),
                    ))
                    .clicked()
                {
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
                ui.label(egui::RichText::new("Conway's Game of Life.").size(20.0));
            });
        });
    }
    fn draw_highlife_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_highlife.texture_id(ctx),
                        self.thumbnail_highlife.size_vec2(),
                    ))
                    .clicked()
                {
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
                ui.label(egui::RichText::new("HighLife(23/36)").size(20.0));
            });
        });
    }
    fn draw_generalized_lifegame_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_generalized_lifegame.texture_id(ctx),
                        self.thumbnail_generalized_lifegame.size_vec2(),
                    ))
                    .clicked()
                    && GeneralizedLifeGameRule::is_valid_rule(&self.life_game_rule)
                {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        self.life_game_rule.clone(),
                        Box::new(App::<
                            8,
                            MooreNeighborhood,
                            GeneralizedLifeGameRule,
                            SquareGrid<LifeGameState>,
                        >::new(
                            GeneralizedLifeGameRule::from_rule(&self.life_game_rule)
                        )),
                    ));
                }
                ui.label(egui::RichText::new("Generalized Lifegame").size(20.0));
                ui.horizontal_wrapped(|ui| {
                    ui.label("rule `{survive}/{birth}` (e.g. 23/3)");
                    ui.add(egui::TextEdit::singleline(&mut self.life_game_rule));
                });
            });
        });
    }
    fn draw_hexlife_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_hexlife.texture_id(ctx),
                        self.thumbnail_hexlife.size_vec2(),
                    ))
                    .clicked()
                    && GeneralizedLifeGameRule::is_valid_rule(&self.life_game_rule)
                {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        self.life_game_rule.clone(),
                        Box::new(App::<
                            6,
                            HexGridNeighborhood,
                            GeneralizedLifeGameRule,
                            HexGrid<LifeGameState>,
                        >::new(
                            GeneralizedLifeGameRule::from_rule(&self.life_game_rule)
                        )),
                    ));
                }
                ui.label(egui::RichText::new("HexLife").size(20.0));
                ui.horizontal_wrapped(|ui| {
                    ui.label("rule `{survive}/{birth}` (e.g. 23/3)");
                    let _ = ui.add(egui::TextEdit::singleline(&mut self.life_game_rule));
                });
            });
        });
    }
    fn draw_wireworld_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_wireworld.texture_id(ctx),
                        self.thumbnail_wireworld.size_vec2(),
                    ))
                    .clicked()
                {
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
                ui.label(egui::RichText::new("WireWorld").size(20.0));
            });
        });
    }
    fn draw_grayscott_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_gray_scott.texture_id(ctx),
                        self.thumbnail_gray_scott.size_vec2(),
                    ))
                    .clicked()
                {
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
                ui.label(egui::RichText::new("Gray-Scott").size(20.0));
            });
        });
    }
    fn draw_dynamic_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        // TODO
                        self.thumbnail_lifegame.texture_id(ctx),
                        self.thumbnail_lifegame.size_vec2(),
                    ))
                    .clicked()
                {
                    self.focus = Some(self.apps.len());
                    let app = App::<8, MooreNeighborhood, DynamicRule, SquareGrid<DynamicState>>{
                            fix_board_size: true,
                            ..Default::default()
                        };
                    self.apps.push(("User Defined".to_string(), Box::new(app)));
                }
                ui.label(egui::RichText::new("User-Defined").size(20.0));
            });
        });
    }

    fn draw_card(&mut self, idx: usize, ctx: &egui::Context, ui: &mut egui::Ui) {
        match idx {
            0 => self.draw_lifegame_card(ctx, ui),
            1 => self.draw_highlife_card(ctx, ui),
            2 => self.draw_generalized_lifegame_card(ctx, ui),
            3 => self.draw_hexlife_card(ctx, ui),
            4 => self.draw_wireworld_card(ctx, ui),
            5 => self.draw_grayscott_card(ctx, ui),
            6 => self.draw_dynamic_card(ctx, ui),
            _ => (),
        }
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
                let region = ui.clip_rect();
                let regsize = region.max - region.min;
                let n_card_x = (regsize.x / self.card_width).floor() as usize; // margin?
                let n_card_x = if n_card_x == 0 { 1 } else { n_card_x };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut idx = 0;
                    while idx < 7 {
                        ui.horizontal(|ui| {
                            for _ in 0..n_card_x {
                                if 7 <= idx {
                                    break;
                                }
                                self.draw_card(idx, ctx, ui);
                                idx += 1;
                            }
                        });
                    }
                });
            });
        }
    }
}
