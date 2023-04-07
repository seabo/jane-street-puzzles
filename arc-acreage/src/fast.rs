//! A faster implementation of the backtracking search algorithm to generate curves of a desired
//! area.
//!
//! This algorithm is only capable of generating curves of integer area. The search space is
//! dramatically reduced by treating quarter circle arcs as straight line diagonal segments. Now,
//! there are only two possible loop segments at each point along the loop, rather than the four
//! possibilities when using curved quarter circle arcs. This reduces the branching factor in the
//! search algorithm from 6 to 3. Each time we find a valid loop, it contributes (2n choose n)
//! possibilities to the total count of closed curves built out of quarter circle segments (where
//! 2n is the total path length). Each straight line segment in the configuration this algorithm
//! discovers can maintain the same integer area when translated to the space of quarter-circle if
//! and only half of the segments are converted to contributors of '1-π/4' area, and the other half
//! are converted to contributors of 'π/4' area.
//!
//! This algorithm is so much faster than the original version that we can afford to completely
//! relax the search constraints and it will still produce the result in under a second. This gives
//! us even more confidence in the accuracy of our answer.

/// A cell in the grid.
///
/// The non-empty cells have diagonal slants in them, either forward-facing (╱) or backward-facing
/// (╲).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Forward,
    Backward,
}

/// Representation of an area enclosed by a closed curve in the grid.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Area {
    /// The number of full units.
    pub units: u8,
    /// The number of half units (contributed by a slanted path segment).
    pub half: u8,
}

impl Area {
    /// Simplify the `Area` representation by treating pairs of half segments as full units.
    pub fn simplify(&self) -> Self {
        let u = self.half / 2;

        Area {
            units: self.units + u,
            half: self.half - 2 * u,
        }
    }

    /// Whether this is an integer area of `n` units.
    #[allow(dead_code)]
    pub fn is_integer(&self, n: u8) -> bool {
        let simplified = self.simplify();
        simplified.units == n && simplified.half == 0
    }
}

impl std::fmt::Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.simplify();

        write!(f, "{}", a.units as f32 + a.half as f32 * 0.5)
    }
}

/// An error returned when attempting to calculate the area enclosed by a loop in a `Grid`.
#[derive(Debug)]
pub enum AreaError {
    LoopNotClosed,
}

/// A 7x7 grid, containing empty cells and curve segments.
#[derive(Clone, Debug)]
pub struct Grid {
    data: [[Cell; 7]; 7],
}

impl Grid {
    /// Create a new `Grid` from an array of arrays of `Cell`s.
    pub fn new(data: [[Cell; 7]; 7]) -> Self {
        Self { data }
    }

    /// Calculate the enclosed area inside the loop drawn in this `Grid`. This function assumes
    /// that the shape passed is a valid closed loop. It does not check this.
    pub fn loop_area(&self) -> Result<Area, AreaError> {
        // These should sum to exactly 49 at the end of looping through the grid.
        let mut n = 0; // The number of slanted segments encountered.
        let mut k = 0; // The number of outside full cells encountered.
        let mut j = 0; // The number of inside full cells encountered.
        let mut h = 0; // The number of segments which contribute a half unit of enclosed area.

        for row in &self.data {
            // Tracking whether we are inside or outside the loop before we inspect this cell.
            let mut outside = true;

            for col in row {
                use Cell::*;
                match col {
                    Empty => {
                        if outside {
                            k += 1;
                        } else {
                            j += 1;
                        }
                    }
                    Forward | Backward => {
                        n += 1;
                        h += 1;

                        outside = !outside;
                    }
                }
            }
        }

        if n + k + j != 49 {
            Err(AreaError::LoopNotClosed)
        } else {
            Ok(Area { units: j, half: h }.simplify())
        }
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.data {
            for col in row {
                use Cell::*;
                match *col {
                    Empty => write!(f, "·")?,
                    Forward => write!(f, "╱")?,
                    Backward => write!(f, "╲")?,
                };
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

/// A data structure for generating closed loops of a target area, using a back-tracking algorithm.
#[derive(Debug)]
pub struct Generator {
    /// The target area we are aiming for.
    target: Area,
    /// The maximum number of inner cells (i.e. not part of the outer boundary of the grid) we can
    /// have forming part of the curve. This constraint is useful to prune a very large number of
    /// search paths, assuming we can prove it rigorously for our desired target area.
    max_inner_cells: u8,
    /// The maximum length of the loop (in segments). This constraint is useful to prune some search
    /// paths, assuming we can prove it rigorously for our target area.
    max_length: u8,
    /// The current state of the grid.
    grid: Grid,
    /// Whether we have placed something in each cell of the grid so far during the backtracking
    /// algorithm.
    placed: [[bool; 7]; 7],
    /// Tracks the number of placed cells; used to ensure backtracking doesn't recurse forever.
    placed_cnt: u8,
    /// The order of placements made in the grid. When we backtrack, we pop off elements and undo
    /// those moves. The first tuple is the coordinate of the cell being placed. The second element
    /// is the coordinates of the head before we placed this move (for undoing).
    moves: Vec<((u8, u8), (u8, u8))>,
    /// The coordinates of the loop's starting point, used to determine when we have closed the
    /// loop. Coordinates are on the grid lines, zero-indexed from the top-left of the grid.
    start: (u8, u8),
    /// The location of the head of the loop we are generating. Coordinates are on the grid lines.
    head: (u8, u8),
    /// Storage for all the valid grids we find.
    valid_grids: Vec<Grid>,
    /// Counter of all valid grids, capturing the multiplicity. This algorithm will find valid
    /// _layouts_ using forward/backward strokes. Each of these has associated with it a large
    /// number of grids drawn with quarter circle arcs. In fact, if the path length is 2n (it must
    /// be even), then there are (2n choose n) arc-segment paths for each path we find.
    valid_cnt: usize,
    calls: usize,
    /// The number of cells we have placed not on the outer rim of the grid. This constraint is
    /// useful to prune a large number of search paths, assuming we can prove it rigorously for our
    /// target area.
    inner_cells: usize,
}

impl Generator {
    /// Create a new `Generator`.
    pub fn new(target: Area, max_inner_cells: u8, max_length: u8) -> Self {
        Self {
            target: target.simplify(),
            max_inner_cells,
            max_length,
            grid: Grid::new([[Cell::Empty; 7]; 7]),
            placed: [[false; 7]; 7],
            placed_cnt: 0,
            moves: Vec::with_capacity(49),
            start: (0, 0),
            head: (0, 0),
            valid_grids: Vec::new(),
            valid_cnt: 0,
            calls: 0,
            inner_cells: 0,
        }
    }

    /// Generate the total count of valid grids (including multiplicity), and a vec of all the grid
    /// layouts.
    pub fn generate(mut self) -> (usize, Vec<Grid>) {
        self.next_cell();
        (self.valid_cnt, self.valid_grids)
    }

    fn next_cell(&mut self) {
        self.calls += 1;
        if self.calls % 1_000_000 == 0 {
            println!(
                "{} nodes visited; {} valid grids found",
                self.calls,
                self.valid_grids.len(),
            );
        }

        if self.moves.len() == 0 {
            // Try every possibility for the first cell.
            for r in 0..7 {
                for c in 0..7 {
                    use Cell::*;
                    for cell in [Forward, Backward] {
                        match cell {
                            Empty => unreachable!(),
                            Forward => {
                                let start = (r + 1, c);
                                if start == (0, 0)
                                    || start == (0, 7)
                                    || start == (7, 0)
                                    || start == (7, 7)
                                {
                                    continue;
                                }

                                self.head = (r, c + 1);
                                self.start = (r + 1, c);
                                self.place(r, c, cell, r, c + 1);
                            }
                            Backward => {
                                let start = (r, c);
                                if start == (0, 0)
                                    || start == (0, 7)
                                    || start == (7, 0)
                                    || start == (7, 7)
                                {
                                    continue;
                                }

                                self.head = (r + 1, c + 1);
                                self.start = (r, c);
                                self.place(r, c, cell, r + 1, c + 1);
                            }
                        }

                        self.next_cell();
                        self.unplace();
                    }

                    // Unlike with non-first cells, we want to maintain the flag that marks
                    // this as placed, because we don't want the loop to ever come back here.
                    self.placed[r as usize][c as usize] = true;
                    assert_eq!(self.grid.data, [[Empty; 7]; 7]);
                }
            }
        } else {
            // Get the last cell that we placed.
            let ((pr, pc), _) = self.moves.last().expect("should be non-empty").clone();
            let p_cell = self.grid.data[pr as usize][pc as usize];
            assert_ne!(p_cell, Cell::Empty);

            let mut moves = Vec::with_capacity(3);

            // Consider the current head. There are four cells surrounding it. Establish from the
            // `placed` grid which of these we can move to next.
            let (hr, hc) = self.head;

            for dr in [-1, 1] {
                let nr = hr as i32 + dr;
                if nr < 0 || nr > 7 {
                    continue;
                }
                let nr = nr as u8;

                for dc in [-1, 1] {
                    let nc = hc as i32 + dc;
                    if nc < 0 || nc > 7 {
                        continue;
                    }
                    let nc = nc as u8;

                    let ncellr = if dr == 1 { nr - 1 } else { nr };
                    let ncellc = if dc == 1 { nc - 1 } else { nc };

                    // Check that the proposed new cell location isn't already populated.
                    if self.placed[ncellr as usize][ncellc as usize] {
                        continue;
                    }

                    // Push the relevant move into the list.
                    use Cell::*;
                    match (dr, dc) {
                        (-1, -1) | (1, 1) => {
                            moves.push((ncellr, ncellc, Backward, nr, nc));
                        }
                        (-1, 1) | (1, -1) => {
                            moves.push((ncellr, ncellc, Forward, nr, nc));
                        }
                        _ => unreachable!(),
                    }
                }
            }

            // Iterate the moves
            for (ncellr, ncellc, n_cell, nr, nc) in moves {
                // Check if the current possibility causes a self-intersection. If so, continue.
                let mut c = 0_u8;

                // Top-left
                if nr > 0
                    && nc > 0
                    && self.grid.data[nr as usize - 1][nc as usize - 1] != Cell::Empty
                {
                    c += 1;
                }
                // Top-right
                if nr > 0 && nc < 7 && self.grid.data[nr as usize - 1][nc as usize] != Cell::Empty {
                    c += 1;
                }
                // Bottom-left
                if nr < 7 && nc > 0 && self.grid.data[nr as usize][nc as usize - 1] != Cell::Empty {
                    c += 1;
                }
                // Bottom-right
                if nr < 7 && nc < 7 && self.grid.data[nr as usize][nc as usize] != Cell::Empty {
                    c += 1;
                }

                if c >= 2 {
                    continue;
                }

                // Check if this possibility closes the loop. If so, add it to the valid grids.
                // The current `placed_cnt` must have odd parity if adding this possibility would
                // close the loop, because a closed loop must have even parity.
                if nr == self.start.0 && nc == self.start.1 {
                    assert_eq!(c, 1);

                    self.place(ncellr, ncellc, n_cell, nr, nc);
                    assert!(self.placed_cnt % 2 == 0);

                    let area = self.grid.loop_area().expect("we formed a loop").simplify();

                    if area == self.target {
                        self.valid_grids.push(self.grid.clone());
                        self.valid_cnt += central_binom(self.placed_cnt / 2);

                        self.unplace();
                    } else {
                        // We formed a loop, but it was the wrong size.
                        self.unplace();
                        continue;
                    }
                }

                if self.placed_cnt + 1 > self.max_length {
                    continue;
                }

                // Place the current possibility
                self.place(ncellr, ncellc, n_cell, nr, nc);

                if self.inner_cells <= self.max_inner_cells as usize {
                    self.next_cell();
                }

                self.unplace();
            }
        }
    }

    fn place(&mut self, row: u8, col: u8, c: Cell, headr: u8, headc: u8) {
        let cell = &mut self.grid.data[row as usize][col as usize];
        let placed = &mut self.placed[row as usize][col as usize];

        assert_eq!(*placed, false);

        *cell = c;
        *placed = true;
        self.placed_cnt += 1;
        self.moves.push(((row, col), self.head));
        self.head = (headr, headc);

        if row > 0 && row < 6 && col > 0 && col < 6 {
            self.inner_cells += 1;
        }
    }

    fn unplace(&mut self) {
        let ((row, col), old_head) = self
            .moves
            .pop()
            .expect("should never call `unplace` with nothing to unplace");
        let cell = &mut self.grid.data[row as usize][col as usize];
        let placed = &mut self.placed[row as usize][col as usize];
        assert_eq!(*placed, true);

        *cell = Cell::Empty;
        *placed = false;
        self.placed_cnt -= 1;
        self.head = old_head;

        if row > 0 && row < 6 && col > 0 && col < 6 {
            self.inner_cells -= 1;
        }
    }
}

/// Returns the value of 2n choose n, the central binomial coefficient. Implemented as const lookup
/// table for speed and ease.
///
/// <https://oeis.org/A000984>
///
/// # Panics
///
/// Panics for values of n > 26.
const fn central_binom(n: u8) -> usize {
    match n {
        0 => 1,
        1 => 2,
        2 => 6,
        3 => 20,
        4 => 70,
        5 => 252,
        6 => 924,
        7 => 3432,
        8 => 12870,
        9 => 48620,
        10 => 184756,
        11 => 705432,
        12 => 2704156,
        13 => 10400600,
        14 => 40116600,
        15 => 155117520,
        16 => 601080390,
        17 => 2333606220,
        18 => 9075135300,
        19 => 35345263800,
        20 => 137846528820,
        21 => 538257874440,
        22 => 2104098963720,
        23 => 8233430727600,
        24 => 32247603683100,
        25 => 126410606437752,
        26 => 495918532948104,
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_shapes_have_correct_area() {
        use Cell::*;

        let grid1 = Grid::new([
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Forward, Backward, Empty, Empty, Empty, Empty],
            [Empty, Backward, Forward, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
        ]);

        assert_eq!(
            grid1.loop_area().unwrap().simplify(),
            Area { units: 2, half: 0 }
        );

        let grid2 = Grid::new([
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Forward, Backward, Forward, Backward, Empty],
            [Empty, Empty, Backward, Empty, Empty, Forward, Empty],
            [Empty, Empty, Empty, Backward, Forward, Empty, Empty],
        ]);

        assert!(grid2.loop_area().unwrap().is_integer(6));

        let grid3 = Grid::new([
            [Empty, Empty, Forward, Backward, Forward, Backward, Empty],
            [Empty, Forward, Empty, Empty, Empty, Empty, Backward],
            [Forward, Empty, Empty, Empty, Empty, Empty, Forward],
            [Backward, Empty, Empty, Empty, Empty, Empty, Backward],
            [Forward, Empty, Empty, Empty, Empty, Empty, Forward],
            [Backward, Empty, Empty, Empty, Empty, Forward, Empty],
            [Empty, Backward, Forward, Backward, Forward, Empty, Empty],
        ]);

        assert_eq!(grid3.loop_area().unwrap(), Area { units: 32, half: 0 });
    }
}
