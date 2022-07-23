use std::vec::Vec;

const CHUNK_LEN: usize = 16;
const CHUNK_SIZE: usize = CHUNK_LEN * CHUNK_LEN;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Dead,
    Alive,
}

impl std::default::Default for State {
    fn default() -> Self {
        State::Dead
    }
}

impl State {
    fn flip(&mut self) {
        if *self == State::Dead {
            *self = State::Alive;
        } else {
            *self = State::Dead;
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    cells: [State; CHUNK_SIZE],
}

impl std::default::Default for Chunk {
    fn default() -> Self {
        Self {
            cells: [State::default(); CHUNK_SIZE],
        }
    }
}

impl Chunk {
    fn cell_at(&self, x: usize, y: usize) -> Option<State> {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        self.cells.get(y * CHUNK_LEN + x).copied()
    }
    fn cell_at_mut(&mut self, x: usize, y: usize) -> Option<&mut State> {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        self.cells.get_mut(y * CHUNK_LEN + x)
    }
}

#[derive(Default)]
pub struct Board {
    num_chunks_x: usize,
    num_chunks_y: usize,
    chunks: Vec<Chunk>,
    buffer: Vec<Chunk>,
}

impl Board {
    fn new(x_chunks: usize, y_chunks: usize) -> Self {
        Self {
            num_chunks_x: x_chunks,
            num_chunks_y: y_chunks,
            chunks: vec![Chunk::default(); x_chunks * y_chunks],
            buffer: vec![Chunk::default(); x_chunks * y_chunks],
        }
    }

    fn width(&self) -> usize {
        self.num_chunks_x * CHUNK_LEN
    }
    fn height(&self) -> usize {
        self.num_chunks_y * CHUNK_LEN
    }

    fn n_chunks_x(&self) -> usize {
        self.num_chunks_x
    }
    fn n_chunks_y(&self) -> usize {
        self.num_chunks_y
    }

    fn chunk_at(&self, x: usize, y: usize) -> &Chunk {
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

    fn cell_at(&self, x: usize, y: usize) -> Option<State> {
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
    fn cell_at_mut(&mut self, x: usize, y: usize) -> Option<&mut State> {
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

    fn bufcell_at_mut(&mut self, x: usize, y: usize) -> &mut State {
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
        self.buffer[chy * self.num_chunks_x + chx]
            .cell_at_mut(clx, cly)
            .expect("bufcell_at_mut always succeed")
    }

    fn expand_x(&mut self, n: isize) {
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
    fn expand_y(&mut self, n: isize) {
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

    fn update(&mut self) {
        // inside
        for j in 1..self.height().saturating_sub(1) {
            for i in 1..self.width().saturating_sub(1) {
                let mut nalive = 0;
                for ny in j - 1..=j + 1 {
                    for nx in i - 1..=i + 1 {
                        if self.cell_at(nx, ny) == Some(State::Alive) {
                            nalive += 1;
                        }
                    }
                }
                let self_is_alive = self.cell_at(i, j) == Some(State::Alive);

                let buf = self.bufcell_at_mut(i, j);
                *buf = if nalive == 3 || (self_is_alive && nalive == 4) {
                    State::Alive
                } else {
                    State::Dead
                };
            }
        }
        // edges
        {
            let j = 0;
            for i in 0..self.width() {
                let buf = self.bufcell_at_mut(i, j);
                *buf = State::Dead;
            }
        }
        if self.height() > 1 {
            let j = self.height() - 1;
            for i in 0..self.width() {
                let buf = self.bufcell_at_mut(i, j);
                *buf = State::Dead;
            }
        }
        {
            let i = 0;
            for j in 0..self.height() {
                let buf = self.bufcell_at_mut(i, j);
                *buf = State::Dead;
            }
        }
        if self.width() > 1 {
            let i = self.width() - 1;
            for j in 0..self.height() {
                let buf = self.bufcell_at_mut(i, j);
                *buf = State::Dead;
            }
        }
        std::mem::swap(&mut self.chunks, &mut self.buffer);
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    running: bool,
    grid_width: f32,
    #[serde(skip)]
    board: Board,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            board: Board::new(8, 8),
            grid_width: 32.0,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn min_gridsize() -> f32 {
        8.0
    }
    pub fn max_gridsize() -> f32 {
        128.0
    }
    pub fn scroll_factor() -> f32 {
        16.0
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    #[rustfmt::skip] // keep whitespace to align
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.running {
            self.board.update();
        }

        // do not remove this block, to avoid dead lock around ctx
        {
            let scroll = ctx.input().scroll_delta.y / Self::scroll_factor();
            self.grid_width = (self.grid_width + scroll)
                .clamp(Self::min_gridsize(), Self::max_gridsize()).ceil();
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            let min_grid = Self::min_gridsize();
            let max_grid = Self::max_gridsize();
            ui.add(egui::Slider::new(&mut self.grid_width, min_grid..=max_grid).text("grid_width"));

            ui.label(format!("current cells: {}x{}", self.board.width(), self.board.height()));
            ui.label(format!("current chunks: {}x{}", self.board.n_chunks_x(), self.board.n_chunks_y()));

            ui.toggle_value(&mut self.running, "Run");

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

            // draw grid
            let region = painter.clip_rect();
            // clear region
            painter.add(epaint::RectShape::filled(
                egui::Rect {
                    min: region.min,
                    max: region.max,
                },
                egui::Rounding::none(),
                egui::Color32::from_rgb(0, 255, 0),
            ));

            // determine the number of chunks
            let regsize = region.max - region.min;
            let delta = self.grid_width.ceil();
            let rdelta = 1.0 / delta;
            let nx = (regsize.x * rdelta).ceil() as usize;
            let ny = (regsize.y * rdelta).ceil() as usize;

            let n_chunks_x = nx / CHUNK_LEN + (if nx % CHUNK_LEN == 0 { 0 } else { 1 });
            let n_chunks_y = ny / CHUNK_LEN + (if ny % CHUNK_LEN == 0 { 0 } else { 1 });
            let chunks_dx = n_chunks_x.saturating_sub(self.board.num_chunks_x);
            let chunks_dy = n_chunks_y.saturating_sub(self.board.num_chunks_y);

            self.board.expand_x(chunks_dx as isize);
            self.board.expand_y(chunks_dy as isize);

            // change state by clicking
            {
                for ev in ctx.input().events.iter() {
                    if let egui::Event::PointerButton{pos, button, pressed, modifiers} = ev {
                        let _ = modifiers;
                        if *pressed { continue; }
                        if *button != egui::PointerButton::Primary { continue; }
                        let dxy = *pos - region.min;
                        if dxy.x < 0.0 || dxy.y < 0.0 { continue; }

                        let ix = (dxy.x * rdelta).floor() as usize;
                        let iy = (dxy.y * rdelta).floor() as usize;
                        if let Some(cell) = self.board.cell_at_mut(ix, iy) {
                            cell.flip();
//                             *cell = State::Alive;
                        }
                    }
                }
            }

            // draw grid
            let ofs = if delta <= 25.0 { 0.0 } else { 1.0 };
            for j in 0..ny {
                let y0 =  j    as f32 * delta + region.min.y + ofs;
                let y1 = (j+1) as f32 * delta + region.min.y - ofs;
                for i in 0..nx {
                    let x0 =  i    as f32 * delta + region.min.x + ofs;
                    let x1 = (i+1) as f32 * delta + region.min.x - ofs;

                    if self.board.cell_at(i, j).unwrap_or(State::Dead) == State::Dead {
                        painter.add(epaint::RectShape::filled(
                            egui::Rect {
                                min: egui::Pos2 { x: x0, y: y0 },
                                max: egui::Pos2 { x: x1, y: y1 },
                            },
                            egui::Rounding::none(),
                            egui::Color32::from_rgb(0, 0, 0),
                        ));
                    }
                }
            }

            egui::warn_if_debug_build(ui);
        });
    }
}
