/// # Board
///
/// Board contains a set of cells as a square lattice.
/// Board is a set of Chunks and Chunk is a set of Cells.
/// Since cell state type varies depending on its rule, both of them are generic.
///
/// Currently, miniascape supports Square and Hexagonal lattice.
///
/// Since we can re-interpret hexagonal lattice as a square lattice in the
/// following way, we use the same `Board` and `Chunk` implementation in both
/// `SquareGrid` and `HexGrid`.
///
/// ```ignore
/// //   .-- x
/// //   |
/// //   y   .'.   .'.   .'.   .'.   .'.   .'.   .'.
/// //     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.
/// // 0   | 0,0 | 1,0 | 2,0 | 3,0 | 4,0 |     |N-1,0|
/// //     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
/// //       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
/// // 1      | 0,1 | 1,1 | 2,1 | 3,1 |     |     |N-1,1|
/// //       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
/// //     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
/// // 2   | 0,2 | 1,2 | 2,2 | 3,2 | 4,2 |     |N-1,2|
/// //     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
/// //       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
/// // 3      | 0,3 | 1,3 | 2,3 | 3,3 |     |     |     |
/// //       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
/// //     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
/// // 4   | 0,4 |     |     |     |     |     |     |
/// //     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
/// //       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
/// // 5      | 0,5 |     |     |     |     |     |     |
/// //       .'.   .'.   .'.   .'.   .'.   .'.   .'.   .'
/// //     .'   `.'   `.'   `.'   `.'   `.'   `.'   `.'
/// // 6   |     |     |     |     |     |     |     |
/// //     '.   .'.   .'.   .'.   .'.   .'.   .'.   .'.
/// //       `.'   `.'   `.'   `.'   `.'   `.'   `.'   `.
/// // 7      |N-1,0|     |     |     |     |     |N-1,N-1|
/// //        '.   .'.   .'.   .'.   .'.   .'.   .'.   .'
/// //          `.'   `.'   `.'   `.'   `.'   `.'   `.'
/// //
/// ```
use crate::rule::{Rule, State};
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) const CHUNK_LEN: usize = 16;
pub(crate) const CHUNK_SIZE: usize = CHUNK_LEN * CHUNK_LEN;

/// A square-shaped Chunk of cells.
#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk<T: State> {
    #[serde(with = "serde_arrays")]
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    cells: [T; CHUNK_SIZE],
}

impl<T: State> std::default::Default for Chunk<T> {
    fn default() -> Self {
        Self { cells: array_init::array_init(|_| Default::default()) }
    }
}

impl<T: State> Chunk<T> {
    /// We need to take initial value because the default value depends on rule.
    fn init(i: T) -> Self {
        Self { cells: array_init::array_init(|_| i.clone()) }
    }

    /// access to a cell with (x, y) coordinate.
    fn cell_at(&self, x: usize, y: usize) -> &T {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        &self.cells[y * CHUNK_LEN + x]
    }
    /// mut access to a cell with (x, y) coordinate.
    fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        assert!(x < CHUNK_LEN && y < CHUNK_LEN, "x = {}, y = {}", x, y);
        &mut self.cells[y * CHUNK_LEN + x]
    }
    /// set the state of all the cells in this chunk as default.
    fn clear<R>(&mut self, rule: &R) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
    {
        for c in self.cells.iter_mut() {
            *c = rule.default_state()?;
        }
        Ok(())
    }
    /// randomize the state of all the cells in this chunk.
    fn randomize<R, Rn>(&mut self, rule: &R, rng: &mut Rn) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
        Rn: Rng,
    {
        for c in self.cells.iter_mut() {
            *c = rule.randomize(rng)?;
        }
        Ok(())
    }
}

/// A square lattice of chunks.
#[derive(Default, Serialize, Deserialize)]
pub struct Grid<T: State> {
    pub(crate) num_chunks_x: usize,
    pub(crate) num_chunks_y: usize,
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    pub(crate) chunks: Vec<Chunk<T>>,
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    pub(crate) buffer: Vec<Chunk<T>>,
}

impl<T: State> Grid<T> {
    pub fn init(x_chunks: usize, y_chunks: usize, i: T) -> Self {
        Self {
            num_chunks_x: x_chunks,
            num_chunks_y: y_chunks,
            chunks: vec![Chunk::init(i.clone()); x_chunks * y_chunks],
            buffer: vec![Chunk::init(i); x_chunks * y_chunks],
        }
    }

    /// The number of Cells, not chunks.
    pub fn width(&self) -> usize {
        self.num_chunks_x * CHUNK_LEN
    }
    /// The number of Cells, not chunks.
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
    pub(crate) fn chunk_len(&self) -> usize {
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
    pub fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.chunks, &mut self.buffer);
    }

    pub fn expand_x(&mut self, n: isize, init: T) {
        if n == 0 {
            return;
        }

        let na = n.unsigned_abs();
        let mut new_chunks = Vec::new();
        new_chunks.resize((self.num_chunks_x + na) * self.num_chunks_y, Chunk::init(init.clone()));

        let x_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = j * (self.num_chunks_x + na) + (i + x_ofs);
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer
            .resize((self.num_chunks_x + na) * self.num_chunks_y, Chunk::init(init));
        self.num_chunks_x += na;
    }
    pub fn expand_y(&mut self, n: isize, init: T) {
        if n == 0 {
            return;
        }

        let na = n.unsigned_abs();
        let mut new_chunks = Vec::new();
        new_chunks.resize(self.num_chunks_x * (self.num_chunks_y + na), Chunk::init(init.clone()));

        let y_ofs = if 0 <= n { 0 } else { na };
        for j in 0..self.num_chunks_y {
            for i in 0..self.num_chunks_x {
                let idx = (j + y_ofs) * (self.num_chunks_x) + i;
                new_chunks[idx] = self.chunk_at(i, j).clone();
            }
        }
        self.chunks = new_chunks;
        self.buffer
            .resize(self.num_chunks_x * (self.num_chunks_y + na), Chunk::init(init));
        self.num_chunks_y += na;
    }

    pub fn clear<R>(&mut self, rule: &R) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
    {
        for ch in self.chunks.iter_mut() {
            ch.clear(rule)?;
        }
        Ok(())
    }
    pub fn randomize<R, Rn>(&mut self, rule: &R, rng: &mut Rn) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
        Rn: Rng,
    {
        for ch in self.chunks.iter_mut() {
            ch.randomize(rule, rng)?;
        }
        Ok(())
    }
}

/// We have (currently) two different boards, `SquareGrid` and `HexGrid`.
/// To use both with the same Rule, we need to make interface the same.
///
/// Most of the functions are actually implemented in `Grid`.
/// Visualization and UI functions are the only difference.
///
pub trait Board<T: State> {
    fn init(x_chunks: usize, y_chunks: usize, default_state: T) -> Self;
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn n_chunks_x(&self) -> usize;
    fn n_chunks_y(&self) -> usize;
    fn has_chunk(&self, x: usize, y: usize) -> bool;
    fn chunk_len(&self) -> usize;
    fn chunk_width_px(&self, cell_width: f32) -> f32;
    fn chunk_height_px(&self, cell_height: f32) -> f32;
    fn width_px(&self, cell_width: f32) -> f32;
    fn height_px(&self, cell_height: f32) -> f32;

    fn chunk_at(&self, x: usize, y: usize) -> &Chunk<T>;

    fn has_cell(&self, x: usize, y: usize) -> bool;
    fn cell_at(&self, x: usize, y: usize) -> &T;
    fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T;

    fn bufcell_at_mut(&mut self, x: usize, y: usize) -> &mut T;
    fn swap_buffer(&mut self);

    fn expand_x(&mut self, n: isize, init: T);
    fn expand_y(&mut self, n: isize, init: T);

    fn clear<R: Rule<CellState = T>>(&mut self, rule: &R) -> anyhow::Result<()>;
    fn randomize<R, Rn>(&mut self, rule: &R, rng: &mut Rn) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
        Rn: Rng;

    /// it takes a cell coordinate `(x, y)` and returns the center position of
    /// the corresponding cell.
    fn location(
        &self,
        x: usize,
        y: usize,
        origin: egui::Pos2,
        region_min: egui::Pos2,
        cell_width: f32,
    ) -> egui::Pos2;

    /// takes a clicked position, returns the corresponding cell coordinate.
    fn clicked(&self, x: f32, y: f32, cell_width: f32) -> Option<(usize, usize)>;

    /// visualize the board.
    fn paint<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,
        alpha: f32,
    ) -> anyhow::Result<()>;

    #[allow(clippy::too_many_arguments)]
    fn paint_clipboard<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,

        xofs: usize,
        yofs: usize,
        cb: &ClipBoard<T>,
        alpha: f32,
    ) -> anyhow::Result<()>;

    fn paste_clipboard(
        &mut self,
        xofs: usize,
        yofs: usize,
        cb: &ClipBoard<T>,
    ) -> anyhow::Result<()>;
}

/// Square grid wraps a `Grid` and implement vis/UI functions.
#[derive(Default, Serialize, Deserialize)]
pub struct SquareGrid<T: State> {
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    grid: Grid<T>,
}
impl<T: State> Board<T> for SquareGrid<T> {
    fn init(x_chunks: usize, y_chunks: usize, ini: T) -> Self {
        Self { grid: Grid::init(x_chunks, y_chunks, ini) }
    }

    fn width(&self) -> usize {
        self.grid.num_chunks_x * CHUNK_LEN
    }
    fn height(&self) -> usize {
        self.grid.num_chunks_y * CHUNK_LEN
    }

    fn n_chunks_x(&self) -> usize {
        self.grid.n_chunks_x()
    }
    fn n_chunks_y(&self) -> usize {
        self.grid.n_chunks_y()
    }
    fn has_chunk(&self, x: usize, y: usize) -> bool {
        self.grid.has_chunk(x, y)
    }
    fn chunk_len(&self) -> usize {
        self.grid.chunk_len()
    }

    fn chunk_width_px(&self, cell_width: f32) -> f32 {
        CHUNK_LEN as f32 * cell_width
    }
    fn chunk_height_px(&self, cell_height: f32) -> f32 {
        CHUNK_LEN as f32 * cell_height
    }

    fn width_px(&self, cell_width: f32) -> f32 {
        self.grid.width() as f32 * cell_width
    }
    fn height_px(&self, cell_height: f32) -> f32 {
        self.grid.height() as f32 * cell_height
    }

    fn chunk_at(&self, x: usize, y: usize) -> &Chunk<T> {
        self.grid.chunk_at(x, y)
    }

    fn has_cell(&self, x: usize, y: usize) -> bool {
        self.grid.has_cell(x, y)
    }
    fn cell_at(&self, x: usize, y: usize) -> &T {
        self.grid.cell_at(x, y)
    }
    fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.grid.cell_at_mut(x, y)
    }

    fn bufcell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.grid.bufcell_at_mut(x, y)
    }
    fn swap_buffer(&mut self) {
        self.grid.swap_buffer();
    }

    fn expand_x(&mut self, n: isize, init: T) {
        self.grid.expand_x(n, init)
    }
    fn expand_y(&mut self, n: isize, init: T) {
        self.grid.expand_y(n, init)
    }

    fn clear<R: Rule<CellState = T>>(&mut self, rule: &R) -> anyhow::Result<()> {
        self.grid.clear(rule)
    }
    fn randomize<R, Rn>(&mut self, rule: &R, rng: &mut Rn) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
        Rn: Rng,
    {
        self.grid.randomize(rule, rng)
    }

    fn location(
        &self,
        x: usize,
        y: usize,
        origin: egui::Pos2,
        region_min: egui::Pos2,
        cell_width: f32,
    ) -> egui::Pos2 {
        let x = (x as f32 + 0.5) * cell_width - origin.x + region_min.x;
        let y = (y as f32 + 0.5) * cell_width - origin.y + region_min.y;

        egui::Pos2 { x, y }
    }

    fn clicked(&self, x: f32, y: f32, cell_width: f32) -> Option<(usize, usize)> {
        let x = x / cell_width;
        let y = y / cell_width;
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let x = x.floor() as usize;
        let y = y.floor() as usize;
        if self.grid.width() < x || self.grid.height() < y {
            None
        } else {
            Some((x, y))
        }
    }

    fn paint<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,
        alpha: f32,
    ) -> anyhow::Result<()> {
        let alpha = (256.0 * alpha).clamp(0.0, 255.0) as u8;
        let region = painter.clip_rect();
        let regsize = region.max - region.min;

        // clear region
        painter.add(epaint::RectShape::filled(
            egui::Rect { min: region.min, max: region.max },
            egui::Rounding::none(),
            rule.background(),
        ));

        let rwidth = 1.0_f32 / cell_width;
        let cell_begin_x = (origin.x * rwidth).floor() as usize;
        let cell_begin_y = (origin.y * rwidth).floor() as usize;
        let cell_end_x = ((origin.x + regsize.x) * rwidth).ceil() as usize;
        let cell_end_y = ((origin.y + regsize.y) * rwidth).ceil() as usize;

        // draw grid
        let ofs = if cell_width <= 16.0 { 0.0 } else { 1.0 };
        for j in cell_begin_y..cell_end_y {
            let y0 = j as f32 * cell_width - origin.y + region.min.y + ofs;
            let y1 = (j + 1) as f32 * cell_width - origin.y + region.min.y - ofs;

            for i in cell_begin_x..cell_end_x {
                let x0 = i as f32 * cell_width - origin.x + region.min.x + ofs;
                let x1 = (i + 1) as f32 * cell_width - origin.x + region.min.x - ofs;

                if !self.grid.has_cell(i, j) {
                    continue;
                }
                let color = rule.color(self.grid.cell_at(i, j))?;
                let color =
                    egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha);
                painter.add(epaint::RectShape::filled(
                    egui::Rect {
                        min: egui::Pos2 { x: x0, y: y0 },
                        max: egui::Pos2 { x: x1, y: y1 },
                    },
                    egui::Rounding::none(),
                    color,
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_clipboard<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,

        xofs: usize,
        yofs: usize,
        clip_board: &ClipBoard<R::CellState>,
        alpha: f32,
    ) -> anyhow::Result<()> {
        let region = painter.clip_rect();
        let alpha = (256.0 * alpha).clamp(0.0, 255.0) as u8;

        let cell_begin_x = xofs;
        let cell_begin_y = yofs;
        let cell_end_x = xofs + clip_board.width();
        let cell_end_y = yofs + clip_board.height();

        // draw grid
        let ofs = if cell_width <= 16.0 { 0.0 } else { 1.0 };
        for j in cell_begin_y..cell_end_y {
            let y0 = j as f32 * cell_width - origin.y + region.min.y + ofs;
            let y1 = (j + 1) as f32 * cell_width - origin.y + region.min.y - ofs;

            for i in cell_begin_x..cell_end_x {
                let x0 = i as f32 * cell_width - origin.x + region.min.x + ofs;
                let x1 = (i + 1) as f32 * cell_width - origin.x + region.min.x - ofs;

                let i = i - xofs;
                let j = j - yofs;
                if !clip_board.has_cell(i, j) {
                    continue;
                }
                if let Some(cell) = clip_board.cell_at(i, j) {
                    let color = rule.color(cell)?;
                    let color = egui::Color32::from_rgba_premultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        alpha,
                    );

                    painter.add(epaint::RectShape::filled(
                        egui::Rect {
                            min: egui::Pos2 { x: x0, y: y0 },
                            max: egui::Pos2 { x: x1, y: y1 },
                        },
                        egui::Rounding::none(),
                        color,
                    ));
                }
            }
        }
        Ok(())
    }

    fn paste_clipboard(
        &mut self,
        xofs: usize,
        yofs: usize,
        cb: &ClipBoard<T>,
    ) -> anyhow::Result<()> {
        self.grid.paste_clipboard(xofs, yofs, cb)
    }
}

/// Hex grid wraps a `Grid` and implement vis/UI functions.
#[derive(Default, Serialize, Deserialize)]
pub struct HexGrid<T: State> {
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    grid: Grid<T>,
}
impl<T: State> Board<T> for HexGrid<T> {
    fn init(x_chunks: usize, y_chunks: usize, ini: T) -> Self {
        Self { grid: Grid::init(x_chunks, y_chunks, ini) }
    }

    fn width(&self) -> usize {
        self.grid.num_chunks_x * CHUNK_LEN
    }
    fn height(&self) -> usize {
        self.grid.num_chunks_y * CHUNK_LEN
    }

    fn n_chunks_x(&self) -> usize {
        self.grid.n_chunks_x()
    }
    fn n_chunks_y(&self) -> usize {
        self.grid.n_chunks_y()
    }
    fn has_chunk(&self, x: usize, y: usize) -> bool {
        self.grid.has_chunk(x, y)
    }
    fn chunk_len(&self) -> usize {
        self.grid.chunk_len()
    }

    fn chunk_width_px(&self, cell_width: f32) -> f32 {
        CHUNK_LEN as f32 * cell_width
    }
    fn chunk_height_px(&self, cell_height: f32) -> f32 {
        CHUNK_LEN as f32 * cell_height * 3.0_f32.sqrt() * 0.5
    }

    fn width_px(&self, cell_width: f32) -> f32 {
        self.grid.width() as f32 * cell_width
    }
    fn height_px(&self, cell_height: f32) -> f32 {
        self.grid.height() as f32 * cell_height * 3.0_f32.sqrt() * 0.5
    }

    fn chunk_at(&self, x: usize, y: usize) -> &Chunk<T> {
        self.grid.chunk_at(x, y)
    }

    fn has_cell(&self, x: usize, y: usize) -> bool {
        self.grid.has_cell(x, y)
    }
    fn cell_at(&self, x: usize, y: usize) -> &T {
        self.grid.cell_at(x, y)
    }
    fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.grid.cell_at_mut(x, y)
    }

    fn bufcell_at_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.grid.bufcell_at_mut(x, y)
    }
    fn swap_buffer(&mut self) {
        self.grid.swap_buffer();
    }

    fn expand_x(&mut self, n: isize, init: T) {
        self.grid.expand_x(n, init)
    }
    fn expand_y(&mut self, n: isize, init: T) {
        self.grid.expand_y(n, init)
    }

    fn clear<R: Rule<CellState = T>>(&mut self, rule: &R) -> anyhow::Result<()> {
        self.grid.clear(rule)
    }
    fn randomize<R, Rn>(&mut self, rule: &R, rng: &mut Rn) -> anyhow::Result<()>
    where
        R: Rule<CellState = T>,
        Rn: Rng,
    {
        self.grid.randomize(rule, rng)
    }

    fn location(
        &self,
        x: usize,
        y: usize,
        origin: egui::Pos2,
        region_min: egui::Pos2,
        cell_width: f32,
    ) -> egui::Pos2 {
        let diameter = cell_width;
        let r = cell_width * 0.5_f32;

        let cy = r + y as f32 * r * 3.0_f32.sqrt() - origin.y + region_min.y;
        let xofs = if y % 2 == 0 { r } else { diameter };
        let cx = xofs + (x as f32) * diameter - origin.x + region_min.x;

        egui::Pos2 { x: cx, y: cy }
    }

    fn clicked(&self, x: f32, y: f32, cell_width: f32) -> Option<(usize, usize)> {
        let diameter = cell_width;
        let r = diameter * 0.5;
        if y < 0.0 {
            return None;
        }
        let y = (y / (r * 3.0_f32.sqrt())).floor() as usize;
        let x = (if y % 2 == 0 { x } else { x - r }) / diameter;
        if x < 0.0 {
            return None;
        }
        let x = x.floor() as usize;

        if self.grid.width() < x || self.grid.height() < y {
            None
        } else {
            Some((x, y))
        }
    }

    fn paint<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,
        alpha: f32,
    ) -> anyhow::Result<()> {
        let alpha = (256.0 * alpha).clamp(0.0, 255.0) as u8;
        let region = painter.clip_rect();
        let regsize = region.max - region.min;

        // clear region
        painter.add(epaint::RectShape::filled(
            egui::Rect { min: region.min, max: region.max },
            egui::Rounding::none(),
            rule.background(),
        ));

        let diameter = cell_width;
        let r = diameter * 0.5;
        let sqrt3 = 3.0_f32.sqrt();

        let rwidth_x = 1.0_f32 / diameter;
        let rwidth_y = 1.0_f32 / (r * sqrt3);
        let cell_begin_x = (origin.x * rwidth_x).floor() as usize;
        let cell_begin_y = (origin.y * rwidth_y).floor() as usize;
        let cell_end_x = ((origin.x + regsize.x) * rwidth_x).ceil() as usize;
        let cell_end_y = ((origin.y + regsize.y) * rwidth_y).ceil() as usize;

        // draw circles
        for j in cell_begin_y..cell_end_y {
            let y = r + j as f32 * r * sqrt3 - origin.y + region.min.y;
            let xofs = if j % 2 == 0 { r } else { diameter };

            for i in cell_begin_x..cell_end_x {
                if !self.grid.has_cell(i, j) {
                    continue;
                }
                let x = xofs + (i as f32) * diameter - origin.x + region.min.x;

                let color = rule.color(self.grid.cell_at(i, j))?;
                let color =
                    egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha);
                painter.add(epaint::CircleShape::filled(egui::Pos2 { x, y }, r, color));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_clipboard<R: Rule<CellState = T>>(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
        rule: &R,

        xofs: usize,
        yofs: usize,
        clip_board: &ClipBoard<R::CellState>,
        alpha: f32,
    ) -> anyhow::Result<()> {
        let region = painter.clip_rect();
        let alpha = (256.0 * alpha).clamp(0.0, 255.0) as u8;

        let cell_begin_x = xofs;
        let cell_begin_y = yofs;
        let cell_end_x = xofs + clip_board.width();
        let cell_end_y = yofs + clip_board.height();

        let diameter = cell_width;
        let r = diameter * 0.5;
        let sqrt3 = 3.0_f32.sqrt();

        // draw circles
        for j in cell_begin_y..cell_end_y {
            let y = r + j as f32 * r * sqrt3 - origin.y + region.min.y;
            let offset = if j % 2 == 0 { r } else { diameter };

            for i in cell_begin_x..cell_end_x {
                let x = offset + (i as f32) * diameter - origin.x + region.min.x;

                let i = i - xofs;
                let j = j - yofs;

                if !clip_board.has_cell(i, j) {
                    continue;
                }
                if let Some(cell) = clip_board.cell_at(i, j) {
                    let color = rule.color(cell)?;
                    let color = egui::Color32::from_rgba_premultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        alpha,
                    );

                    painter.add(epaint::CircleShape::filled(egui::Pos2 { x, y }, r, color));
                }
            }
        }
        Ok(())
    }

    fn paste_clipboard(
        &mut self,
        xofs: usize,
        yofs: usize,
        cb: &ClipBoard<T>,
    ) -> anyhow::Result<()> {
        self.grid.paste_clipboard(xofs, yofs, cb)
    }
}

/// A small piece of board to copy-paste a region in a board
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClipBoard<T: State> {
    x: usize,
    y: usize,
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    cells: Vec<Option<T>>,
}

impl<T: State> ClipBoard<T> {
    pub fn new(x: usize, y: usize) -> Self {
        let mut cells = Vec::new();
        cells.resize(x * y, None);
        Self { x, y, cells }
    }
    pub fn from_vec(x: usize, y: usize, cells: Vec<Option<T>>) -> Option<Self> {
        if cells.len() == x * y {
            Some(Self { x, y, cells })
        } else {
            None
        }
    }

    fn cell_at(&self, x: usize, y: usize) -> &Option<T> {
        assert!(x < self.x && y < self.y, "x({}) < {} && y({}) < {}", x, self.x, y, self.y);
        &self.cells[x + y * self.x]
    }
    pub fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut Option<T> {
        assert!(x < self.x && y < self.y, "x({}) < {} && y({}) < {}", x, self.x, y, self.y);
        &mut self.cells[x + y * self.x]
    }

    pub fn has_cell(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }

    pub fn width(&self) -> usize {
        self.x
    }
    pub fn height(&self) -> usize {
        self.y
    }

    pub fn rotate(&mut self) {
        let mut rotated = Self::new(self.y, self.x);
        for j in 0..rotated.height() {
            for i in 0..rotated.width() {
                *rotated.cell_at_mut(i, j) = self.cell_at(self.x - 1 - j, i).clone();
            }
        }
        *self = rotated;
    }
}

#[derive(Error, Debug)]
struct ClipBoardError {
    msg: String,
}
impl std::fmt::Display for ClipBoardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl<T: State> Grid<T> {
    fn paste_clipboard(
        &mut self,
        xofs: usize,
        yofs: usize,
        cb: &ClipBoard<T>,
    ) -> anyhow::Result<()> {
        for j in 0..cb.height() {
            for i in 0..cb.width() {
                if let Some(c) = cb.cell_at(i, j) {
                    *self.cell_at_mut(i + xofs, j + yofs) = c.clone();
                }
            }
        }
        Ok(())
    }
}
