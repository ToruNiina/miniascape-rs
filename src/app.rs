use crate::board::{Board, ClipBoard, CHUNK_LEN};
use crate::rule::{Neighbors, Rule, State};

use anyhow::anyhow;
use anyhow::Context as _;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

// TODO: We derive Deserialize/Serialize so we can persist app state on shutdown.

/// An application to manage a cell automaton.
///
/// Several application can run at the same time but only the focused app will
/// be updated its state and others will be paused.
///
pub struct App<N: Neighbors, R: Rule<N>, B: Board<N, R>> {
    pub(crate) rule: R,
    pub(crate) board: B,
    pub(crate) fix_board_size: bool,
    pub(crate) fix_grid_size: bool,
    pub(crate) click_mode: ClickMode,
    pub(crate) running: bool,
    pub(crate) inspector: Option<(usize, usize)>,
    pub(crate) inspector_code_buf: String,
    pub(crate) grid_width: f32,
    pub(crate) origin: egui::Pos2,
    pub(crate) grabbed: bool,
    pub(crate) cell_modifying: Option<R::CellState>,
    pub(crate) rng: rand::rngs::StdRng,
    pub(crate) err: Option<String>,
    pub(crate) serdes_buffer: Option<String>,
    pub(crate) cursor_is_on_sidepanel: bool, // at the last frame

    pub(crate) clipboard: Option<ClipBoard<R::CellState>>,
    pub(crate) secondary_start: Option<(usize, usize)>,
    pub(crate) secondary_curr: Option<(usize, usize)>,
    pub(crate) selected_region: Option<((usize, usize), (usize, usize))>,
}

// in some cases, like PC trackpad + browser, gestures cannot be used.
// as a fallback system, we introduce click mode.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum ClickMode {
    Normal,
    Grab,
    Inspect,
}

impl<N: Neighbors, R: Rule<N>, B: Board<N, R>> Default for App<N, R, B> {
    fn default() -> Self {
        let rule = R::default();
        let mut board = B::new(4, 3);
        board.clear(&rule).expect("default construction must not fail");
        Self {
            rule,
            board,
            fix_board_size: false,
            fix_grid_size: false,
            click_mode: ClickMode::Normal,
            running: false,
            inspector: None,
            inspector_code_buf: String::new(),
            grid_width: 32.0,
            origin: egui::Pos2::new(0.0, 0.0),
            grabbed: false,
            cell_modifying: None,
            rng: rand::rngs::StdRng::seed_from_u64(123456789),
            err: None,
            serdes_buffer: None,
            cursor_is_on_sidepanel: false,
            clipboard: None,
            secondary_start: None,
            secondary_curr: None,
            selected_region: None,
        }
    }
}

pub struct Clicked {
    primary: Option<(usize, usize)>,
    secondary: Option<(usize, usize)>,
}
impl Clicked {
    fn new(primary: Option<(usize, usize)>, secondary: Option<(usize, usize)>) -> Self {
        Self { primary, secondary }
    }
}

impl<N: Neighbors, R: Rule<N>, B> App<N, R, B>
where
    for<'de> B: Board<N, R> + Deserialize<'de>,
{
    pub fn new(rule: R) -> Self {
        let mut board = B::new(4, 3);
        board.clear(&rule).expect("default construction must not fail");
        Self {
            rule,
            board,
            click_mode: ClickMode::Normal,
            inspector_code_buf: String::new(),
            grid_width: 32.0,
            origin: egui::Pos2::new(0.0, 0.0),
            rng: rand::rngs::StdRng::seed_from_u64(123456789),
            ..Default::default()
        }
    }
    pub fn min_gridsize() -> f32 {
        1.0
    }
    pub fn max_gridsize() -> f32 {
        128.0
    }
    pub fn scroll_factor() -> f32 {
        1.0 / 128.0
    }

    /// Detect which cell is clicked.
    ///
    /// If no button is pressed or pressed position is out of board, it returns `NotClicked`.
    /// Otherwise, it returns which cell is clicked.
    pub fn clicked(&self, ctx: &egui::Context, region_min: egui::Pos2) -> Clicked {
        let pointer = &ctx.input().pointer;
        if !pointer.primary_down() && !pointer.secondary_down() {
            return Clicked::new(None, None);
        }

        let pos = pointer
            .interact_pos()
            .unwrap_or(egui::Pos2::new(-f32::INFINITY, -f32::INFINITY));

        let dx = pos.x - region_min.x + self.origin.x;
        let dy = pos.y - region_min.y + self.origin.y;

        if let Some((ix, iy)) = self.board.clicked(dx, dy, self.grid_width) {
            let p = if pointer.primary_down() { Some((ix, iy)) } else { None };
            let s = if pointer.secondary_down() { Some((ix, iy)) } else { None };
            Clicked::new(p, s)
        } else {
            Clicked::new(None, None)
        }
    }

    fn load_from_dropped_file(&mut self, ctx: &egui::Context) -> anyhow::Result<()> {
        let dropped_files = ctx.input().raw.dropped_files.clone();
        if dropped_files.is_empty() {
            return Ok(());
        }
        if let Some(file) = dropped_files.iter().find(|f| f.name.ends_with(".json")) {
            if let Some(bytes) = &file.bytes {
                let content = std::str::from_utf8(bytes)
                    .context(format!("Couldn't read file content as utf8 -> {}", file.name))?
                    .to_owned();
                self.board = serde_json::from_str(&content)
                    .context(format!("Couldn't load file content as board -> {}", file.name))?;
                Ok(())
            } else {
                Err(anyhow!("file {} could not read", file.name))
            }
        } else {
            Err(anyhow!(
                "only json deserializaion is supported. file \"{:?}\" ignored",
                dropped_files.into_iter().map(|f| f.name).collect::<Vec<String>>()
            ))
        }
    }
}

impl<N: Neighbors, R: Rule<N>, B> eframe::App for App<N, R, B>
where
    for<'de> B: Board<N, R> + Serialize + Deserialize<'de>,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //         eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            if let Err(e) = self.board.update(&self.rule) {
                self.err = Some(format!("{:?}", e));
            }
        }

        let sidepanel_response = egui::SidePanel::left("side_panel")
            .show(ctx, |ui| {
                ui.push_id(0, |ui| {
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
                });

                ui.separator(); // -------------------------------------------------

                ui.horizontal_wrapped(|ui| {
                    ui.toggle_value(&mut self.running, "Run");

                    if ui.button("Step").clicked() {
                        if let Err(e) = self.board.update(&self.rule) {
                            self.err = Some(format!("{:?}", e));
                        }
                        ui.ctx().request_repaint();
                    }
                    if ui.button("Reset").clicked() {
                        if let Err(e) = self.board.clear(&self.rule) {
                            self.err = Some(format!("{:?}", e));
                        }
                    }
                    if ui.button("Randomize").clicked() {
                        if let Err(e) = self.board.randomize(&self.rule, &mut self.rng) {
                            self.err = Some(format!("{:?}", e));
                        }
                    }
                });

                ui.separator(); // -------------------------------------------------

                if ui.button("serialize").clicked() {
                    self.serdes_buffer =
                        Some(serde_json::to_string(&self.board).expect("TODO: serde"));
                }
                if let Some(sb) = self.serdes_buffer.as_mut() {
                    let mut open = true;
                    egui::Window::new("Serialize").open(&mut open).show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(sb)
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    });
                    if !open {
                        self.serdes_buffer = None;
                    }
                }

                ui.separator(); // -------------------------------------------------

                let min_grid = Self::min_gridsize();
                let max_grid = Self::max_gridsize();
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.grid_width, min_grid..=max_grid)
                            .text("grid_width"),
                    );
                    ui.checkbox(&mut self.fix_grid_size, "Fix Grid Size");
                });
                ui.checkbox(&mut self.fix_board_size, "Fix Board Size");

                ui.label("On browser, PC trackpad does not work. Instead, change click mode.");
                ui.radio_value(&mut self.click_mode, ClickMode::Normal, "Normal mode");
                ui.radio_value(&mut self.click_mode, ClickMode::Grab, "Grab mode");
                ui.radio_value(&mut self.click_mode, ClickMode::Inspect, "Inspect mode");

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

                for (name, clip) in self.rule.library().into_iter() {
                    if ui.button(name).clicked() {
                        self.clipboard = Some(clip)
                    }
                }

                // we can only know the cursor hovers on sidepanel after drawing
                // sidepanel, so we use the status of the last frame
                if let Err(e) = self.rule.ui(ui, ctx, self.cursor_is_on_sidepanel) {
                    self.err = Some(format!("{:?}", e));
                }
            })
            .response;

        self.cursor_is_on_sidepanel = sidepanel_response.hovered();

        // TODO: deserialize if cursor is on central panel
        if !self.cursor_is_on_sidepanel {
            if let Err(e) = self.load_from_dropped_file(ctx) {
                self.err = Some(format!("{:?}", e));
            }
        }

        if !self.cursor_is_on_sidepanel {
            if let Some(multi_touch) = ctx.multi_touch() {
                if self.grabbed {
                    self.origin -= multi_touch.translation_delta;
                }
                if multi_touch.num_touches == 2 || self.click_mode == ClickMode::Grab {
                    self.grabbed = true;
                } else {
                    self.grabbed = false;
                }
            } else {
                // we need to drop pointer after checking the value to release ctx.
                let pointer = &ctx.input().pointer;
                if self.grabbed {
                    self.origin -= pointer.delta();
                }
                if pointer.middle_down()
                    || (self.click_mode == ClickMode::Grab && pointer.any_down())
                {
                    self.grabbed = true;
                } else {
                    self.grabbed = false;
                }
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
            // zoom in/out

            if !self.fix_grid_size {
                if let Some(multi_touch) = ctx.multi_touch() {
                    if multi_touch.zoom_delta < 0.99 || 1.01 < multi_touch.zoom_delta {
                        let new_grid_width = (self.grid_width * multi_touch.zoom_delta)
                            .clamp(Self::min_gridsize(), Self::max_gridsize())
                            .ceil();

                        let magnification = new_grid_width / self.grid_width;
                        let center = self.origin.to_vec2() + (regsize * 0.5);

                        self.origin = (center * magnification - regsize * 0.5).to_pos2();
                        self.grid_width = new_grid_width;
                    }
                } else {
                    // we need to drop scroll after checking it to release ctx
                    let scroll = ctx.input().scroll_delta.y * Self::scroll_factor();
                    if !self.cursor_is_on_sidepanel && scroll != 0.0 {
                        let new_grid_width = (self.grid_width * 1.1_f32.powf(scroll))
                            .clamp(Self::min_gridsize(), Self::max_gridsize())
                            .ceil();

                        let magnification = new_grid_width / self.grid_width;
                        let center = self.origin.to_vec2() + (regsize * 0.5);

                        self.origin = (center * magnification - regsize * 0.5).to_pos2();
                        self.grid_width = new_grid_width;
                    }
                }
            }

            // ----------------------------------------------------------------
            // expand board size if needed

            if !self.fix_board_size {
                let chunk_pxls_x = self.board.chunk_width_px(delta);
                let chunk_pxls_y = self.board.chunk_height_px(delta);

                let default_state = self.rule.default_state();
                if let Ok(init) = default_state {
                    if self.origin.x < 0.0 {
                        let d = (self.origin.x / chunk_pxls_x).floor();
                        self.board.expand_x(d as isize, init.clone());
                        self.origin.x -= chunk_pxls_x * d;
                        assert!(0.0 <= self.origin.x);
                    }
                    if self.board.width_px(delta) <= self.origin.x + regsize.x {
                        let dx = self.origin.x + regsize.x - self.board.width_px(delta);
                        assert!(0.0 <= dx);
                        let d = (dx / chunk_pxls_x).ceil();
                        self.board.expand_x(d as isize, init.clone());
                    }

                    if self.origin.y < 0.0 {
                        let d = (self.origin.y / chunk_pxls_y).floor();
                        self.board.expand_y(d as isize, init.clone());
                        self.origin.y -= chunk_pxls_y * d;
                        assert!(0.0 <= self.origin.y);
                    }
                    if self.board.height_px(delta) <= self.origin.y + regsize.y {
                        let dy = self.origin.y + regsize.y - self.board.height_px(delta);
                        assert!(0.0 <= dy);
                        let d = (dy / chunk_pxls_y).ceil();
                        self.board.expand_y(d as isize, init);
                    }
                } else {
                    let e = default_state.expect_err("already checked");
                    self.err = Some(format!("{:?}", e));
                }
            }

            // ----------------------------------------------------------------
            // draw board to the central panel

            if let Err(e) = self.board.paint(&painter, self.origin, delta, &self.rule, 0.0) {
                self.err = Some(format!("{:?}", e));
            }

            // ----------------------------------------------------------------
            // handle left/right click

            if !self.cursor_is_on_sidepanel {
                let Clicked { primary, secondary } = self.clicked(ctx, region.min);

                // stop running and inspect cell state by right click
                if self.click_mode == ClickMode::Inspect {
                    self.running = false;
                    self.cell_modifying = None;
                    self.inspector = primary.or(secondary);
                } else if secondary.is_some() {
                    self.running = false;
                    self.cell_modifying = None;

                    if self.secondary_start.is_none() {
                        self.secondary_start = secondary;
                    }
                    self.secondary_curr = secondary;

                    let (sx, sy) = self.secondary_start.expect("we have just set this");
                    let (ex, ey) = self.secondary_curr.expect("we have just set this");
                    let min = (sx.min(ex), sy.min(ey));
                    let max = (sx.max(ex), sy.max(ey));
                    self.selected_region = Some((min, max));
                } else if primary.is_some() {
                    self.selected_region = None; // reset
                    self.clipboard = None;
                } else if primary.is_none() && secondary.is_none() {
                    self.cell_modifying = None;

                    if let Some((sx, sy)) = self.secondary_start {
                        // set selected region
                        let (ex, ey) = self
                            .secondary_curr
                            .expect("when secondary_start is some, curr is always some");

                        // if the pointer did not move, open inspector
                        if sx == ex && sy == ey {
                            self.inspector = Some((sx, sy));
                        }
                        // reset because secondary button is released
                        self.secondary_start = None;
                        self.secondary_curr = None;
                    }
                }

                // show selected region
                if let Some(((sx, sy), (ex, ey))) = self.selected_region {
                    if sx == ex && sy == ey {
                        // show the corresponding cell
                        let c = self.board.location(sx, sy, self.origin, region.min, delta);
                        let r = delta * 0.5_f32.sqrt();
                        painter.add(epaint::CircleShape::stroke(
                            c,
                            r,
                            epaint::Stroke { width: 5.0, color: egui::Color32::WHITE },
                        ));
                        painter.add(epaint::CircleShape::stroke(
                            c,
                            r,
                            epaint::Stroke { width: 2.0, color: egui::Color32::BLACK },
                        ));
                    } else {
                        // show the corresponding region

                        let min = self.board.location(sx, sy, self.origin, region.min, delta);
                        let max = self.board.location(ex, ey, self.origin, region.min, delta);
                        let r = delta * 0.5_f32.sqrt();

                        let min = egui::Pos2::new(min.x - r, min.y - r);
                        let max = egui::Pos2::new(max.x + r, max.y + r);

                        painter.add(epaint::RectShape::stroke(
                            epaint::Rect { min, max },
                            epaint::Rounding::same(r),
                            epaint::Stroke { width: 5.0, color: egui::Color32::WHITE },
                        ));
                        painter.add(epaint::RectShape::stroke(
                            epaint::Rect { min, max },
                            epaint::Rounding::same(r),
                            epaint::Stroke { width: 2.0, color: egui::Color32::BLACK },
                        ));
                    }
                }

                // ----------------------------------------------------------------

                if let Some((ix, iy)) = self.inspector {
                    let mut open = true;
                    egui::Window::new("Cell Inspector").open(&mut open).show(ctx, |ui| {
                        self.board.cell_at_mut(ix, iy).inspect(ui, &mut self.inspector_code_buf);
                    });
                    if !open {
                        self.inspector = None;
                        self.selected_region = None;
                    }
                } else if let Some(((sx, sy), (ex, ey))) = self.selected_region {
                    // when copy, cut, or delete is performed, selected region dissapears.
                    let (copy, cut, del) = {
                        let mut input_state = ctx.input_mut();

                        // command on mac, ctrl on others
                        let command = egui::Modifiers::COMMAND;

                        let c = input_state.consume_key(command, egui::Key::C);
                        let x = input_state.consume_key(command, egui::Key::X);
                        let d = input_state.consume_key(egui::Modifiers::NONE, egui::Key::Delete)
                            || input_state.consume_key(egui::Modifiers::NONE, egui::Key::Backspace);
                        (c, x, d)
                    };

                    // copy region to clipboard
                    if copy || cut {
                        let mut cb = ClipBoard::<R::CellState>::new(ex - sx + 1, ey - sy + 1);
                        for j in 0..cb.height() {
                            for i in 0..cb.width() {
                                if self.board.has_cell(sx + i, sy + j) {
                                    *cb.cell_at_mut(i, j) =
                                        Some(self.board.cell_at(sx + i, sy + j).clone());
                                }
                            }
                        }
                        // overwrite
                        self.clipboard = Some(cb);
                    }

                    // clear selected region
                    if cut || del {
                        match self.rule.default_state() {
                            Ok(st) => {
                                for j in sy..=ey {
                                    for i in sx..=ex {
                                        *self.board.cell_at_mut(i, j) = st.clone();
                                    }
                                }
                            }
                            Err(e) => {
                                self.err = Some(format!("{:?}", e));
                            }
                        }
                    }
                } else if let Some((ix, iy)) = primary {
                    // draw cell using `cell_modifying`

                    if let Some(next) = &self.cell_modifying {
                        *self.board.cell_at_mut(ix, iy) = next.clone();
                    } else {
                        let next = self.rule.next(self.board.cell_at(ix, iy).clone());
                        match next {
                            Ok(val) => {
                                *self.board.cell_at_mut(ix, iy) = val.clone();
                                self.cell_modifying = Some(val);
                            }
                            Err(e) => {
                                self.err = Some(format!("{:?}", e));
                            }
                        }
                    }
                }

                // ----------------------------------------------------------------
                // rotate clipboard when R is pressed
                let rot = {
                    let mut input_state = ctx.input_mut();
                    input_state.consume_key(egui::Modifiers::NONE, egui::Key::R)
                };
                if self.clipboard.is_some() && rot {
                    self.clipboard.as_mut().expect("already checked").rotate();
                }

                // ----------------------------------------------------------------
                // paint clipboard on top of current board with alpha

                let cursor_pos = {
                    let pos = &ctx
                        .input()
                        .pointer
                        .interact_pos()
                        .unwrap_or(egui::Pos2::new(-f32::INFINITY, -f32::INFINITY));
                    let dx = pos.x - region.min.x + self.origin.x;
                    let dy = pos.y - region.min.y + self.origin.y;

                    self.board.clicked(dx, dy, self.grid_width)
                };

                if let Some((cursor_x, cursor_y)) = cursor_pos {
                    if let Some(cb) = self.clipboard.as_ref() {
                        let ofs_x = cursor_x - cb.width() / 2;
                        let ofs_y = cursor_y - cb.height() / 2;

                        if let Err(e) = self.board.paint_clipboard(
                            &painter,
                            self.origin,
                            delta,
                            &self.rule,
                            ofs_x,
                            ofs_y,
                            cb,
                            0.5,
                        ) {
                            self.err = Some(format!("{:?}", e));
                        }
                    }

                    let paste = {
                        let mut input_state = ctx.input_mut();
                        input_state.consume_key(egui::Modifiers::COMMAND, egui::Key::V)
                    };
                    if self.clipboard.is_some() && paste {
                        let cb = self.clipboard.as_ref().expect("already checked");
                        let mut ofs_x = (cursor_x as isize) - (cb.width() as isize) / 2;
                        let mut ofs_y = (cursor_y as isize) - (cb.height() as isize) / 2;

                        if let Ok(st) = self.rule.default_state() {
                            // check if clipboard sticks out of the board
                            if ofs_x < 0 {
                                let d = ofs_x.abs() / CHUNK_LEN as isize;
                                let m = ofs_x.abs() % CHUNK_LEN as isize;
                                let n = if m == 0 { d } else { d + 1 };
                                self.board.expand_x(-n, st.clone());
                                ofs_x += n * CHUNK_LEN as isize;
                            }
                            if self.board.width() as isize <= ofs_x + cb.width() as isize {
                                let d = (ofs_x + cb.width() as isize - self.board.width() as isize)
                                    / CHUNK_LEN as isize;
                                let m = (ofs_x + cb.width() as isize - self.board.width() as isize)
                                    % CHUNK_LEN as isize;
                                let n = if m == 0 { d } else { d + 1 };
                                self.board.expand_x(n, st.clone());
                            }

                            if ofs_y < 0 {
                                let d = ofs_y.abs() / CHUNK_LEN as isize;
                                let m = ofs_y.abs() % CHUNK_LEN as isize;
                                let n = if m == 0 { d } else { d + 1 };
                                self.board.expand_y(-n, st.clone());
                                ofs_y += n * CHUNK_LEN as isize;
                            }
                            if self.board.height() as isize <= ofs_y + cb.height() as isize {
                                let d = (ofs_y + cb.height() as isize
                                    - self.board.height() as isize)
                                    / CHUNK_LEN as isize;
                                let m = (ofs_y + cb.height() as isize
                                    - self.board.height() as isize)
                                    % CHUNK_LEN as isize;
                                let n = if m == 0 { d } else { d + 1 };
                                self.board.expand_y(n, st);
                            }

                            // see the current position
                            if let Err(e) =
                                self.board.paste_clipboard(ofs_x as usize, ofs_y as usize, cb)
                            {
                                self.err = Some(format!("{:?}", e));
                            }
                        }
                    }
                }
            }

            // ----------------------------------------------------------------
            // detect debug build
            egui::warn_if_debug_build(ui);
        });

        if let Some(err) = &self.err {
            let mut open = true;
            egui::Window::new("Error Report").open(&mut open).show(ctx, |ui| {
                ui.label(err);
            });
            if !open {
                self.err = None;
            }
        }
    }
}
