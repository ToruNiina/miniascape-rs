use crate::rule::{Neighbors, Rule};
use crate::board::{Board, CHUNK_LEN};
use rand::Rng;

pub trait World {

    type Rule: Rule;
    type Board: Board<<<Self as World>::Rule as Rule>::CellState>;

    fn new(rule: <Self as World>::Rule, x_chunks: usize, y_chunks: usize, z_chunks: usize) -> Self;

    fn rule(&self) -> &Self::Rule;
    fn rule_mut(&mut self) -> &mut Self::Rule;

    fn board(&self) -> &Self::Board;
    fn board_mut(&mut self) -> &mut Self::Board;

    fn current_layer(&self) -> usize;
    fn set_current_layer(&mut self, z: usize);

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn depth(&self) -> usize;

    fn expand_x(&mut self, n: isize, init: <<Self as World>::Rule as Rule>::CellState);
    fn expand_y(&mut self, n: isize, init: <<Self as World>::Rule as Rule>::CellState);
    fn expand_z(&mut self, n: isize, init: <<Self as World>::Rule as Rule>::CellState);

    fn clear(&mut self) -> anyhow::Result<()>;
    fn randomize<Rn: Rng>(&mut self, rng: &mut Rn) -> anyhow::Result<()>;

    /// visualize the slice of the world.
    fn paint(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
    ) -> anyhow::Result<()>;

    fn update(&mut self) -> anyhow::Result<()>;
}

pub struct World2D<R: Rule, B: Board<R::CellState>> {
    rule: R,
    board: B,
}

impl<R: Rule, B: Board<R::CellState>> std::default::Default for World2D<R, B> {
    fn default() -> Self {
        let rule = R::default();
        let init = rule.default_state().unwrap_or_default();
        let mut board = B::init(4, 3, init);
        board.clear(&rule).expect("default construction must not fail");
        Self{rule, board}
    }
}

impl<R, B> World for World2D<R, B>
where
    R: Rule,
    <R as Rule>::Neighborhood: Neighbors,
    B: Board<R::CellState>,
{
    type Rule = R;
    type Board = B;

    fn new(rule: R, x_chunks: usize, y_chunks: usize, z_chunks: usize) -> Self {
        assert!(z_chunks == 1, "World2D has only 1 layer");
        let init = rule.default_state().unwrap_or_default();
        let mut board = B::init(x_chunks, y_chunks, init);
        board.clear(&rule).expect("default construction must not fail");
        Self{rule, board}
    }

    fn rule(&self) -> &R {
        &self.rule
    }
    fn rule_mut(&mut self) -> &mut R {
        &mut self.rule
    }
    fn board(&self) -> &B {
        &self.board
    }
    fn board_mut(&mut self) -> &mut B {
        &mut self.board
    }
    fn current_layer(&self) -> usize {
        0
    }
    fn set_current_layer(&mut self, _z: usize) {
        // do nothing
    }

    fn width(&self) -> usize {
        self.board.width()
    }
    fn height(&self) -> usize {
        self.board.height()
    }
    fn depth(&self) -> usize {
        1
    }

    fn expand_x(&mut self, n: isize, init: R::CellState) {
        self.board.expand_x(n, init);
    }
    fn expand_y(&mut self, n: isize, init: R::CellState) {
        self.board.expand_y(n, init);
    }
    fn expand_z(&mut self, _n: isize, _init: R::CellState) {
        // do nothing
    }

    fn clear(&mut self) -> anyhow::Result<()> {
        self.board.clear(&self.rule)
    }
    fn randomize<Rn: Rng>(&mut self, rng: &mut Rn) -> anyhow::Result<()> {
        self.board.randomize(&self.rule, rng)
    }

    /// visualize the slice of the world.
    fn paint(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        cell_width: f32,
    ) -> anyhow::Result<()> {
        self.board.paint(painter, origin, cell_width, &self.rule, 1.0)
    }

    fn update(&mut self) -> anyhow::Result<()> {
        for _ in 0..self.rule.iteration_per_step() {
            for cj in 0..self.board.n_chunks_y() {
                let y0 = cj * CHUNK_LEN;
                for ci in 0..self.board.n_chunks_x() {
                    let x0 = ci * CHUNK_LEN;
                    for j in 0..CHUNK_LEN {
                        for i in 0..CHUNK_LEN {
                            let x = x0 + i;
                            let y = y0 + j;
                            let idxs = R::Neighborhood::neighbors(
                                x as isize,
                                y as isize,
                                self.width() as isize,
                                self.height() as isize,
                            );

                            *self.board.bufcell_at_mut(x, y) = self.rule.update(
                                self.board.cell_at(x, y).clone(),
                                idxs.into_iter()
                                    .map(|(x, y)| self.board.cell_at(x, y).clone())
                                    .into_iter(),
                            )?;
                        }
                    }
                }
            }
            self.board.swap_buffer();
        }
        Ok(())
    }
}
