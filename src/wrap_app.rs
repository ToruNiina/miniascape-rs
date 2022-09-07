use crate::app::App;
use crate::board::{HexGrid, SquareGrid};
use crate::rule::{HexGridNeighborhood, MooreNeighborhood, VonNeumannNeighborhood};
use crate::world::World2D;

use crate::dynamic_rule::{DynamicRule, DynamicState};
use crate::gray_scott::{GrayScottRule, GrayScottState};
use crate::lifegame::{HighLifeRule, LifeGameRule, LifeGameState, LifeLikeGameRule};
use crate::rock_paper_scissors::{RockPaperScissorsRule, RockPaperScissorsState};
use crate::wireworld::{WireWorldRule, WireWorldState};

use egui_extras::RetainedImage;

#[derive(PartialEq, Eq, std::fmt::Debug)]
enum GridKind {
    Square,
    Hex,
}
#[derive(PartialEq, Eq, std::fmt::Debug)]
enum SquareNeighborKind {
    Moore,
    Neumann,
}

/// An application that manages sub-applications that corresponds to one cell automaton.
///
/// This is the primary app of `miniascape`.
pub struct WrapApp {
    apps: Vec<(String, Box<dyn eframe::App>)>,
    focus: Option<usize>,

    life_game_rule: String,

    dynamic_grid_kind: GridKind,
    dynamic_square_neighbor_kind: SquareNeighborKind,

    rock_paper_scissors_grid_kind: GridKind,
    rock_paper_scissors_square_neighbor_kind: SquareNeighborKind,

    thumbnail_lifegame: RetainedImage,
    thumbnail_lifelike: RetainedImage,
    thumbnail_hexlife: RetainedImage,
    thumbnail_highlife: RetainedImage,
    thumbnail_wireworld: RetainedImage,
    thumbnail_gray_scott: RetainedImage,
    thumbnail_rock_paper_scissors: RetainedImage,

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
            dynamic_grid_kind: GridKind::Square,
            dynamic_square_neighbor_kind: SquareNeighborKind::Moore,
            rock_paper_scissors_grid_kind: GridKind::Square,
            rock_paper_scissors_square_neighbor_kind: SquareNeighborKind::Moore,
            thumbnail_lifegame: RetainedImage::from_image_bytes(
                "thumbnail_lifegame.png",
                include_bytes!("images/thumbnail_lifegame.png"),
            )
            .unwrap(),
            thumbnail_lifelike: RetainedImage::from_image_bytes(
                "thumbnail_lifelike.png",
                include_bytes!("images/thumbnail_lifelike.png"),
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
            thumbnail_rock_paper_scissors: RetainedImage::from_image_bytes(
                "thumbnail_rock_paper_scissors.png",
                include_bytes!("images/thumbnail_rock_paper_scissors.png"),
            )
            .unwrap(),

            card_height: 280.0,
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
                            World2D<LifeGameRule<MooreNeighborhood>, SquareGrid<LifeGameState>>,
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
                            World2D<HighLifeRule<MooreNeighborhood>, SquareGrid<LifeGameState>>,
                        >::default()),
                    ));
                }
                ui.label(egui::RichText::new("HighLife(23/36)").size(20.0));
            });
        });
    }
    fn draw_lifelike_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_lifelike.texture_id(ctx),
                        self.thumbnail_lifelike.size_vec2(),
                    ))
                    .clicked()
                    && LifeLikeGameRule::<MooreNeighborhood>::is_valid_rule(&self.life_game_rule)
                {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        self.life_game_rule.clone(),
                        Box::new(App::<
                            World2D<LifeLikeGameRule<MooreNeighborhood>, SquareGrid<LifeGameState>>,
                        >::new(LifeLikeGameRule::from_rule(
                            &self.life_game_rule,
                        ))),
                    ));
                }
                ui.label(egui::RichText::new("Life-Like").size(20.0));
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
                    && LifeLikeGameRule::<HexGridNeighborhood>::is_valid_rule(&self.life_game_rule)
                {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        self.life_game_rule.clone(),
                        Box::new(App::<
                            World2D<LifeLikeGameRule<HexGridNeighborhood>, HexGrid<LifeGameState>>,
                        >::new(LifeLikeGameRule::from_rule(
                            &self.life_game_rule,
                        ))),
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
                        Box::new(
                            App::<World2D<WireWorldRule, SquareGrid<WireWorldState>>>::default(),
                        ),
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
                        Box::new(
                            App::<World2D<GrayScottRule, SquareGrid<GrayScottState>>>::default(),
                        ),
                    ));
                }
                ui.label(egui::RichText::new("Gray-Scott").size(20.0));
            });
        });
    }
    fn draw_rock_paper_scissors(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::ImageButton::new(
                        self.thumbnail_rock_paper_scissors.texture_id(ctx),
                        self.thumbnail_rock_paper_scissors.size_vec2(),
                    ))
                    .clicked()
                {
                    self.focus = Some(self.apps.len());
                    if self.rock_paper_scissors_grid_kind == GridKind::Square {
                        if self.rock_paper_scissors_square_neighbor_kind
                            == SquareNeighborKind::Moore
                        {
                            self.apps.push((
                                "Rock Paper Scissors".to_string(),
                                Box::new(App::<
                                    World2D<
                                        RockPaperScissorsRule<MooreNeighborhood>,
                                        SquareGrid<RockPaperScissorsState>,
                                    >,
                                >::default()),
                            ));
                        } else {
                            self.apps.push((
                                "Rock Paper Scissors".to_string(),
                                Box::new(App::<
                                    World2D<
                                        RockPaperScissorsRule<VonNeumannNeighborhood>,
                                        SquareGrid<RockPaperScissorsState>,
                                    >,
                                >::default()),
                            ));
                        }
                    } else {
                        self.apps.push((
                            "Rock Paper Scissors".to_string(),
                            Box::new(App::<
                                World2D<
                                    RockPaperScissorsRule<HexGridNeighborhood>,
                                    HexGrid<RockPaperScissorsState>,
                                >,
                            >::default()),
                        ));
                    }
                }
                ui.label(egui::RichText::new("Rock-Paper-Scissors").size(20.0));

                ui.push_id(2, |ui| {
                    egui::ComboBox::from_label("Select Grid")
                        .selected_text(format!("{:?}", self.rock_paper_scissors_grid_kind))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.rock_paper_scissors_grid_kind,
                                GridKind::Square,
                                "Square",
                            );
                            ui.selectable_value(
                                &mut self.rock_paper_scissors_grid_kind,
                                GridKind::Hex,
                                "Hexagonal",
                            );
                        });
                    if self.rock_paper_scissors_grid_kind == GridKind::Square {
                        egui::ComboBox::from_label("Select Neighborhood")
                            .selected_text(format!(
                                "{:?}",
                                self.rock_paper_scissors_square_neighbor_kind
                            ))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.rock_paper_scissors_square_neighbor_kind,
                                    SquareNeighborKind::Moore,
                                    "Moore neighborhood",
                                );
                                ui.selectable_value(
                                    &mut self.rock_paper_scissors_square_neighbor_kind,
                                    SquareNeighborKind::Neumann,
                                    "Von Neumann Neighborhood",
                                );
                            });
                    }
                });
            });
        });
    }
    fn draw_dynamic_card(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(self.card_width);
            ui.set_height(self.card_height);

            ui.vertical_centered(|ui| {
                let thumbnail = if self.dynamic_grid_kind == GridKind::Square {
                    &self.thumbnail_lifegame
                } else {
                    &self.thumbnail_hexlife
                };

                if ui
                    .add(egui::ImageButton::new(thumbnail.texture_id(ctx), thumbnail.size_vec2()))
                    .clicked()
                {
                    self.focus = Some(self.apps.len());
                    if self.dynamic_grid_kind == GridKind::Square {
                        if self.dynamic_square_neighbor_kind == SquareNeighborKind::Moore {
                            let app = App::<
                                World2D<DynamicRule<MooreNeighborhood>, SquareGrid<DynamicState>>,
                            > {
                                fix_board_size: true,
                                ..Default::default()
                            };
                            self.apps.push(("User Defined".to_string(), Box::new(app)));
                        } else {
                            let app = App::<
                                World2D<
                                    DynamicRule<VonNeumannNeighborhood>,
                                    SquareGrid<DynamicState>,
                                >,
                            > {
                                fix_board_size: true,
                                ..Default::default()
                            };
                            self.apps.push(("User Defined".to_string(), Box::new(app)));
                        }
                    } else {
                        let app = App::<
                            World2D<DynamicRule<HexGridNeighborhood>, HexGrid<DynamicState>>,
                        > {
                            fix_board_size: true,
                            ..Default::default()
                        };
                        self.apps.push(("User Defined".to_string(), Box::new(app)));
                    }
                }
                ui.label(egui::RichText::new("User-Defined").size(20.0));

                ui.push_id(1, |ui| {
                    egui::ComboBox::from_label("Select Grid")
                        .selected_text(format!("{:?}", self.dynamic_grid_kind))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.dynamic_grid_kind,
                                GridKind::Square,
                                "Square",
                            );
                            ui.selectable_value(
                                &mut self.dynamic_grid_kind,
                                GridKind::Hex,
                                "Hexagonal",
                            );
                        });
                    if self.dynamic_grid_kind == GridKind::Square {
                        egui::ComboBox::from_label("Select Neighborhood")
                            .selected_text(format!("{:?}", self.dynamic_square_neighbor_kind))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.dynamic_square_neighbor_kind,
                                    SquareNeighborKind::Moore,
                                    "Moore neighborhood",
                                );
                                ui.selectable_value(
                                    &mut self.dynamic_square_neighbor_kind,
                                    SquareNeighborKind::Neumann,
                                    "Von Neumann Neighborhood",
                                );
                            });
                    }
                });
            });
        });
    }

    fn draw_card(&mut self, idx: usize, ctx: &egui::Context, ui: &mut egui::Ui) {
        match idx {
            0 => self.draw_dynamic_card(ctx, ui),
            1 => self.draw_lifegame_card(ctx, ui),
            2 => self.draw_highlife_card(ctx, ui),
            3 => self.draw_lifelike_card(ctx, ui),
            4 => self.draw_hexlife_card(ctx, ui),
            5 => self.draw_wireworld_card(ctx, ui),
            6 => self.draw_grayscott_card(ctx, ui),
            7 => self.draw_rock_paper_scissors(ctx, ui),
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
                    ui.label("source code: ");
                    ui.hyperlink_to("miniascape", "https://github.com/ToruNiina/miniascape-rs");
                    ui.label(", powered by ");
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
                    while idx < 8 {
                        ui.horizontal(|ui| {
                            for _ in 0..n_card_x {
                                if 8 <= idx {
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
