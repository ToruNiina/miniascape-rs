use crate::conway::LifeGameRule;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

/// State of a cell.
///
/// Represents the current state of a cell. To initialize the board, it requires
/// `Clone`. To clear the board, it requires `Default`.
/// The update rule is implemented in `Rule` trait.
pub trait State: Clone + Default {
    fn flip(&mut self); // remove this later; there are more complicated rules
    fn randomize<R: Rng>(&mut self, rng: &mut R);
    fn clear(&mut self);
}

/// Rule of the automaton.
pub trait Rule {
    type CellState: State;

    fn background() -> egui::Color32;
    fn color(st: &Self::CellState) -> egui::Color32;
    fn update(board: &mut Board<Self::CellState>);
}

const CHUNK_LEN: usize = 16;
const CHUNK_SIZE: usize = CHUNK_LEN * CHUNK_LEN;

#[derive(Clone)]
pub struct Chunk<T: State> {
    cells: [T; CHUNK_SIZE],
}

impl<T: State> std::default::Default for Chunk<T> {
    fn default() -> Self {
        Self {
            cells: array_init::array_init(|_| Default::default()),
        }
    }
}

impl<T: State> Chunk<T> {
    fn cell_at(&self, x: usize, y: usize) -> &T {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        &self.cells[y * CHUNK_LEN + x]
    }
    fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        &mut self.cells[y * CHUNK_LEN + x]
    }
    fn clear(&mut self) {
        for c in self.cells.iter_mut() {
            c.clear();
        }
    }
    fn randomize<R: Rng>(&mut self, rng: &mut R) {
        for c in self.cells.iter_mut() {
            c.randomize(rng);
        }
    }
}

#[derive(Default)]
pub struct Board<T: State> {
    pub(crate) num_chunks_x: usize,
    pub(crate) num_chunks_y: usize,
    pub(crate) chunks: Vec<Chunk<T>>,
    pub(crate) buffer: Vec<Chunk<T>>,
}

impl<T: State> Board<T> {
    pub fn new(x_chunks: usize, y_chunks: usize) -> Self {
        Self {
            num_chunks_x: x_chunks,
            num_chunks_y: y_chunks,
            chunks: vec![Chunk::default(); x_chunks * y_chunks],
            buffer: vec![Chunk::default(); x_chunks * y_chunks],
        }
    }

    pub fn width(&self) -> usize {
        self.num_chunks_x * CHUNK_LEN
    }
    pub fn height(&self) -> usize {
        self.num_chunks_y * CHUNK_LEN
    }

    pub(crate) fn n_chunks_x(&self) -> usize {
        self.num_chunks_x
    }
    pub(crate) fn n_chunks_y(&self) -> usize {
        self.num_chunks_y
    }
    pub fn has_chunk(&self, x: usize, y: usize) -> bool {
        x < self.num_chunks_x && y < self.num_chunks_y
    }

    pub(crate) fn chunk_at(&self, x: usize, y: usize) -> &Chunk<T> {
        assert!(
            x < self.num_chunks_x && y < self.num_chunks_y,
            "x = {}, width = {}, y = {}, height = {}",
            x,
            self.num_chunks_x,
            y,
            self.num_chunks_y
        );

        &self.chunks[y * self.num_chunks_x + x]
    }

    pub fn has_cell(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }

    pub fn cell_at(&self, x: usize, y: usize) -> &T {
        assert!(
            x < self.width() && y < self.height(),
            "x = {}, width = {}, y = {}, height = {}",
            x,
            self.width(),
            y,
            self.height()
        );

        let chx = x / CHUNK_LEN;
        let clx = x % CHUNK_LEN;
        let chy = y / CHUNK_LEN;
        let cly = y % CHUNK_LEN;
        self.chunks[chy * self.num_chunks_x + chx].cell_at(clx, cly)
    }
    pub fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        assert!(
            x < self.num_chunks_x * CHUNK_LEN && y < self.num_chunks_y * CHUNK_LEN,
            "x = {}, num_chunks_x = {}, y = {}, num_chunks_y = {}",
            x,
            self.num_chunks_x * CHUNK_LEN,
            y,
            self.num_chunks_y * CHUNK_LEN
        );

        let chx = x / CHUNK_LEN;
        let clx = x % CHUNK_LEN;
        let chy = y / CHUNK_LEN;
        let cly = y % CHUNK_LEN;
        self.chunks[chy * self.num_chunks_x + chx].cell_at_mut(clx, cly)
    }

    pub(crate) fn bufcell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        assert!(
            x < self.width() && y < self.height(),
            "x = {}, width = {}, y = {}, height = {}",
            x,
            self.width(),
            y,
            self.height()
        );

        let chx = x / CHUNK_LEN;
        let clx = x % CHUNK_LEN;
        let chy = y / CHUNK_LEN;
        let cly = y % CHUNK_LEN;
        self.buffer[chy * self.num_chunks_x + chx].cell_at_mut(clx, cly)
    }

    pub fn expand_x(&mut self, n: isize) {
        if n == 0 {
            return;
        }

        let na = n.unsigned_abs();
        let mut new_chunks = Vec::new();
        new_chunks.resize(
            (self.num_chunks_x + na) * self.num_chunks_y,
            Default::default(),
        );

        let x_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = j * (self.num_chunks_x + na) + (i + x_ofs);
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer.resize(
            (self.num_chunks_x + na) * self.num_chunks_y,
            Default::default(),
        );
        self.num_chunks_x += na;
    }
    pub fn expand_y(&mut self, n: isize) {
        if n == 0 {
            return;
        }

        let na = n.unsigned_abs();
        let mut new_chunks = Vec::new();
        new_chunks.resize(
            self.num_chunks_x * (self.num_chunks_y + na),
            Default::default(),
        );

        let y_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = (j + y_ofs) * (self.num_chunks_x) + i;
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer.resize(
            self.num_chunks_x * (self.num_chunks_y + na),
            Default::default(),
        );
        self.num_chunks_y += na;
    }

    pub fn clear(&mut self) {
        for ch in self.chunks.iter_mut() {
            ch.clear();
        }
    }
    pub fn randomize<R: Rng>(&mut self, rng: &mut R) {
        for ch in self.chunks.iter_mut() {
            ch.randomize(rng);
        }
    }

    #[rustfmt::skip] // keep whitespace to align
    pub fn paint(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        bg: egui::Color32,
        coloring: impl Fn(&T) -> egui::Color32,
    ) {
        let region = painter.clip_rect();
        let regsize = region.max - region.min;

        // clear region
        painter.add(epaint::RectShape::filled(
            egui::Rect {
                min: region.min,
                max: region.max,
            },
            egui::Rounding::none(),
            bg,
        ));

        if cell_width < 8.0 {
            let chunk_width = cell_width * CHUNK_LEN as f32;
            let rwidth = 1.0_f32 / chunk_width;
            let chunk_begin_x = (origin.x * rwidth).floor() as usize;
            let chunk_begin_y = (origin.y * rwidth).floor() as usize;
            let chunk_end_x = ((origin.x + regsize.x) * rwidth).ceil() as usize;
            let chunk_end_y = ((origin.y + regsize.y) * rwidth).ceil() as usize;

            let ofs = 1.0;
            for j in chunk_begin_y..chunk_end_y {
                let y0 =  j    as f32 * chunk_width - origin.y + region.min.y + ofs;
                let y1 = (j+1) as f32 * chunk_width - origin.y + region.min.y - ofs;

                for i in chunk_begin_x..chunk_end_x {
                    let x0 =  i    as f32 * chunk_width - origin.x + region.min.x + ofs;
                    let x1 = (i+1) as f32 * chunk_width - origin.x + region.min.x - ofs;

                    if !self.has_chunk(i, j) {
                        continue;
                    }

                    let mut r = 0_u32;
                    let mut g = 0_u32;
                    let mut b = 0_u32;
                    for pxl in self.chunk_at(i, j).cells.iter().map(&coloring) {
                        r += pxl.r() as u32;
                        g += pxl.g() as u32;
                        b += pxl.b() as u32;
                    }
                    r /= CHUNK_SIZE as u32;
                    g /= CHUNK_SIZE as u32;
                    b /= CHUNK_SIZE as u32;

                    painter.add(epaint::RectShape::filled(
                        egui::Rect {
                            min: egui::Pos2 { x: x0, y: y0 },
                            max: egui::Pos2 { x: x1, y: y1 },
                        },
                        egui::Rounding::none(),
                        egui::Color32::from_rgb(r as u8, g as u8, b as u8),
                    ));
                }
            }
        } else {
            let rwidth = 1.0_f32 / cell_width;
            let cell_begin_x = (origin.x * rwidth).floor() as usize;
            let cell_begin_y = (origin.y * rwidth).floor() as usize;
            let cell_end_x = ((origin.x + regsize.x) * rwidth).ceil() as usize;
            let cell_end_y = ((origin.y + regsize.y) * rwidth).ceil() as usize;

            // draw grid
            let ofs = if cell_width <= 25.0 { 0.0 } else { 1.0 };
            for j in cell_begin_y..cell_end_y {
                let y0 =  j    as f32 * cell_width - origin.y + region.min.y + ofs;
                let y1 = (j+1) as f32 * cell_width - origin.y + region.min.y - ofs;

                for i in cell_begin_x..cell_end_x {
                    let x0 =  i    as f32 * cell_width - origin.x + region.min.x + ofs;
                    let x1 = (i+1) as f32 * cell_width - origin.x + region.min.x - ofs;

                    if !self.has_cell(i, j) {
                        continue;
                    }
                    painter.add(epaint::RectShape::filled(
                        egui::Rect {
                            min: egui::Pos2 { x: x0, y: y0 },
                            max: egui::Pos2 { x: x1, y: y1 },
                        },
                        egui::Rounding::none(),
                        coloring(self.cell_at(i, j)),
                    ));
                }
            }
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GenericApp<R: Rule> {
    #[serde(skip)]
    board: Board<R::CellState>,
    #[serde(skip)]
    running: bool,
    #[serde(skip)]
    grid_width: f32,
    #[serde(skip)]
    origin: egui::Pos2,
    #[serde(skip)]
    background: egui::Color32,
    #[serde(skip)]
    grabbed: bool,
    #[serde(skip)]
    clicked: Option<(usize, usize)>,
    #[serde(skip)]
    rng: rand::rngs::StdRng,
}

impl<R: Rule> Default for GenericApp<R> {
    fn default() -> Self {
        Self {
            running: false,
            grid_width: 32.0,
            origin: egui::Pos2::new(0.0, 0.0),
            background: egui::Color32::from_rgb(0, 128, 0),
            grabbed: false,
            board: Board::new(8, 8),
            clicked: None,
            rng: rand::rngs::StdRng::seed_from_u64(123456789),
        }
    }
}

impl<R: Rule> GenericApp<R> {
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
            R::update(&mut self.board);
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            ui.label("left click: flip cell state");
            ui.label("right click: drag board");

            let min_grid = Self::min_gridsize();
            let max_grid = Self::max_gridsize();
            ui.add(egui::Slider::new(&mut self.grid_width, min_grid..=max_grid).text("grid_width"));

            ui.label(format!(
                "current cells: {}x{}",
                self.board.width(),
                self.board.height()
            ));
            ui.label(format!(
                "current chunks: {}x{}",
                self.board.n_chunks_x(),
                self.board.n_chunks_y()
            ));
            ui.label(format!(
                "current origin: ({},{})",
                self.origin.x, self.origin.y
            ));

            ui.toggle_value(&mut self.running, "Run");
            if ui.button("Step").clicked() {
                R::update(&mut self.board);
                ui.ctx().request_repaint();
            }
            if ui.button("Reset").clicked() {
                self.board.clear();
            }
            if ui.button("Randomize").clicked() {
                self.board.randomize(&mut self.rng);
            }
        });

        {
            let pointer = &ctx.input().pointer;
            if self.grabbed {
                self.origin -= pointer.delta();
            }
            if pointer.secondary_down() {
                self.grabbed = true;
            } else {
                self.grabbed = false;
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.running {
                ui.ctx().request_repaint();
            }
            // First make a painter only for inside the region.
            let painter = egui::Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );
            let region = painter.clip_rect();

            // determine the number of chunks after zoom in/out
            let delta = self.grid_width.ceil();
            let rdelta = 1.0 / delta;
            let regsize = region.max - region.min;

            // zoom in/out scroll
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

            // expand board size
            {
                let chunk_pxls = CHUNK_LEN as f32 * delta;

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

            // change state by clicking
            loop {
                // use loop to break from this block later
                let pointer = &ctx.input().pointer;
                if !pointer.primary_down() {
                    self.clicked = None;
                    break;
                }

                let pos = pointer
                    .interact_pos()
                    .unwrap_or(egui::Pos2::new(-f32::INFINITY, -f32::INFINITY));

                let dxy = pos - region.min;
                if dxy.x < 0.0 || dxy.y < 0.0 {
                    self.clicked = None;
                    break;
                }

                let ix = ((dxy.x + self.origin.x) * rdelta).floor() as usize;
                let iy = ((dxy.y + self.origin.y) * rdelta).floor() as usize;
                if self.board.width() <= ix || self.board.height() <= iy {
                    self.clicked = None;
                    break;
                }

                if let Some((x, y)) = self.clicked {
                    if x == ix && y == iy {
                        break;
                    }
                }
                self.board.cell_at_mut(ix, iy).flip();
                self.clicked = Some((ix, iy));
                break;
            }

            // draw board to the central panel
            self.board
                .paint(&painter, self.origin, delta, self.background, R::color);

            // detect debug build
            egui::warn_if_debug_build(ui);
        });
    }
}

#[derive(Default)]
pub struct App {
    apps: Vec<(String, Box<dyn eframe::App>)>,
    focus: Option<usize>,
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
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if ui.selectable_label(self.focus == None, "Home").clicked() {
                        self.focus = None;
                    };
                });

                // list of running apps
                let mut remove = None;
                for (idx, (name, _)) in self.apps.iter().enumerate() {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.selectable_label(self.focus == Some(idx), name).clicked() {
                                self.focus = Some(idx);
                            }
                            if ui.button("🗙").clicked() {
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
                            self.focus = Some(self.apps.len()-1);
                        } else {
                            self.focus = Some(idx);
                        }
                    }
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
                // TODO enable to set up a new app
                if ui.button("start life game").clicked() {
                    self.focus = Some(self.apps.len());
                    self.apps.push((
                        "LifeGame".to_string(),
                        Box::new(GenericApp::<LifeGameRule>::default()),
                    ));
                }
            });
        }
    }
}
