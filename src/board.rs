use crate::rule::{Rule, State, Neighbors};
use rand::Rng;

// ----------------------------------------------------------------------------
//   ___ _             _
//  / __| |_ _  _ _ _ | |__
// | (__| ' \ || | ' \| / /
//  \___|_||_\_,_|_||_|_\_\

const CHUNK_LEN: usize = 16;
const CHUNK_SIZE: usize = CHUNK_LEN * CHUNK_LEN;

#[derive(Clone)]
pub struct Chunk<T: State> {
    cells: [T; CHUNK_SIZE],
}

impl<T: State> std::default::Default for Chunk<T> {
    fn default() -> Self {
        Self { cells: array_init::array_init(|_| Default::default()) }
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

// ----------------------------------------------------------------------------
//  ___                   _
// | _ ) ___  __ _ _ _ __| |
// | _ \/ _ \/ _` | '_/ _` |
// |___/\___/\__,_|_| \__,_|

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
    pub(crate) fn chunk_len() -> usize {
        CHUNK_LEN
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
        new_chunks.resize((self.num_chunks_x + na) * self.num_chunks_y, Default::default());

        let x_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = j * (self.num_chunks_x + na) + (i + x_ofs);
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer
            .resize((self.num_chunks_x + na) * self.num_chunks_y, Default::default());
        self.num_chunks_x += na;
    }
    pub fn expand_y(&mut self, n: isize) {
        if n == 0 {
            return;
        }

        let na = n.unsigned_abs();
        let mut new_chunks = Vec::new();
        new_chunks.resize(self.num_chunks_x * (self.num_chunks_y + na), Default::default());

        let y_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = (j + y_ofs) * (self.num_chunks_x) + i;
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer
            .resize(self.num_chunks_x * (self.num_chunks_y + na), Default::default());
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

    pub fn update<const N: usize, Neighborhood: Neighbors<N>, R>(&mut self, rule: &R)
        where R: Rule<N, Neighborhood, CellState = T>
    {
        for _ in 0..rule.iteration_per_step() {
            for cj in 0..self.n_chunks_y() {
                let y0 = cj * CHUNK_LEN;
                for ci in 0..self.n_chunks_x() {
                    let x0 = ci * CHUNK_LEN;
                    for j in 0..CHUNK_LEN {
                        for i in 0..CHUNK_LEN {
                            let x = x0 + i;
                            let y = y0 + j;
                            let idxs = R::neighbors(x as isize, y as isize);

                            *self.bufcell_at_mut(x, y) = rule.update(
                                    *self.cell_at(x, y),
                                    idxs.map(|(x, y)| {
                                        if x < 0 || y < 0 {
                                            Default::default()
                                        }
                                        let x = x as usize;
                                        let y = y as usize;
                                        if self.has_cell(x, y) {
                                            *self.cell_at(x, y)
                                        } else {
                                            Default::default()
                                        }
                                    }).into_iter()
                                );
                        }
                    }
                }
            }
            std::mem::swap(&mut self.chunks, &mut self.buffer);
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
