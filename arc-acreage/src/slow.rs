//! The original implementation of a backtracking search algorithm to generate closed curves of a
//! desired area.
//!
//! This algorithm builds fully general curves built out of quarter-circle arc segments, and can
//! therefore represent any closed curve as defined in the problem. However, the search function
//! has a large branching factor, since at each step in building out a curve, we can generally
//! transition to any of the 3 adjacent cells, and each of these could have either of two
//! quarter-circle arc segments in it (depending on which way the segment curves). So we usually
//! have a branching factor of 6 at each step, and the algorithm can take a long time to terminate.
//!
//! To help with narrowing the search space, we have included some constraints. It is possible to
//! request the search to ignore curves which have too many segments not on the outer rim of the
//! grid, and also curves above a threshold length. If we can prove constraints that curves of our
//! desired area must obey, then we can use these to reduce the search space.

/// A cell in the grid.
///
/// The non-empty cells have quarter-circle arcs drawn in them, and are denoted by the corner of
/// the cell which contains the quarter-circle segment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Area {
    /// The number of full units.
    pub units: u8,
    /// The number of 1-pi/4 sized units (contributed by a full unit less a quarter circle segment).
    pub small: u8,
    /// The number of pi/4 sized units (contributed by a quarter circle segment).
    pub large: u8,
}

impl Area {
    /// Simplify the `Area` representation by treating pairs of small and large segments as full
    /// units.
    pub fn simplify(&self) -> Self {
        let u = std::cmp::min(self.small, self.large);

        Area {
            units: self.units + u,
            small: self.small - u,
            large: self.large - u,
        }
    }

    /// Whether this is an integer area of `n` units.
    #[allow(dead_code)]
    pub fn is_integer(&self, n: u8) -> bool {
        let simplified = self.simplify();
        simplified.units == n && simplified.small == 0 && simplified.large == 0
    }
}

impl std::fmt::Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.simplify();

        // Integer part. Each 'small' contributes 1-π/4.
        let int = a.units + a.small;
        // Fractional part. Each 'small' contributes a negative, each large a positive.
        let frac = a.large as i8 - a.small as i8;

        write!(f, "{}", int)?;
        if frac == 0 {
            Ok(())
        } else if frac == 1 {
            write!(f, "+π/4")
        } else if frac == -1 {
            write!(f, "-π/4")
        } else if frac < 0 {
            write!(f, "-{}π/4", -frac)
        } else if frac > 0 {
            write!(f, "+{}π/4", frac)
        } else {
            unreachable!()
        }
    }
}

/// An error returned when attempting to calculate the area enclosed by a loop in a `Grid`.
#[derive(Debug)]
pub enum AreaError {
    LoopNotClosed,
}

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
        let mut n = 0; // The number of arc segments encountered.
        let mut k = 0; // The number of outside full cells encountered.
        let mut j = 0; // The number of inside full cells encountered.

        // These should sum to exactly `n` at the end of looping through the grid.
        let mut n_s = 0; // The number of arc segments which contribute a 'small' enclosed area (i.e.
                         // an area of 1-π/4).
        let mut n_b = 0; // The number of arc segments which contribute a 'large' enclosed area (i.e.
                         // an area of +π/4).

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
                    TopLeft | BottomLeft => {
                        n += 1;

                        if outside {
                            n_s += 1;
                        } else {
                            n_b += 1;
                        }

                        outside = !outside;
                    }
                    TopRight | BottomRight => {
                        n += 1;

                        if outside {
                            n_b += 1;
                        } else {
                            n_s += 1;
                        }

                        outside = !outside;
                    }
                }
            }
        }

        assert_eq!(n_s + n_b, n);

        if n + k + j != 49 {
            Err(AreaError::LoopNotClosed)
        } else {
            Ok(Area {
                units: j,
                small: n_s,
                large: n_b,
            }
            .simplify())
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
                    TopLeft => write!(f, "╱")?,
                    TopRight => write!(f, "╲")?,
                    BottomLeft => write!(f, "╲")?,
                    BottomRight => write!(f, "╱")?,
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
    /// search paths, assuming we can prove it rigorously for our target area.
    max_inner_cells: u8,
    /// The maximum length of the loop (in quarter circle arcs). This constraint is useful to prune
    /// some search paths, assuming we can prove it rigorously for our desired target area.
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
            calls: 0,
            inner_cells: 0,
        }
    }

    pub fn generate(mut self) -> Vec<Grid> {
        self.next_cell();
        self.valid_grids
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
                    for cell in [TopLeft, TopRight, BottomLeft, BottomRight] {
                        match cell {
                            Empty => unreachable!(),
                            TopLeft | BottomRight => {
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
                            TopRight | BottomLeft => {
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

            let mut moves = Vec::with_capacity(6);

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

                    // Push the relevant moves into the list.
                    use Cell::*;
                    match (dr, dc) {
                        (-1, -1) | (1, 1) => {
                            moves.push((ncellr, ncellc, TopRight, nr, nc));
                            moves.push((ncellr, ncellc, BottomLeft, nr, nc));
                        }
                        (-1, 1) | (1, -1) => {
                            moves.push((ncellr, ncellc, TopLeft, nr, nc));
                            moves.push((ncellr, ncellc, BottomRight, nr, nc));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_shapes_have_correct_area() {
        use Cell::*;

        let grid1 = Grid::new([
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, TopLeft, TopRight, Empty, Empty, Empty, Empty],
            [Empty, BottomLeft, BottomRight, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
        ]);

        assert_eq!(
            grid1.loop_area().unwrap(),
            Area {
                units: 0,
                small: 4,
                large: 0
            }
        );

        let grid2 = Grid::new([
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, Empty, Empty, Empty, Empty, Empty],
            [Empty, Empty, TopLeft, BottomLeft, TopLeft, TopRight, Empty],
            [Empty, Empty, BottomLeft, Empty, Empty, TopLeft, Empty],
            [Empty, Empty, Empty, TopRight, TopLeft, Empty, Empty],
        ]);

        assert!(grid2.loop_area().unwrap().is_integer(6));

        let grid3 = Grid::new([
            [
                Empty, Empty, TopLeft, BottomLeft, TopLeft, BottomLeft, Empty,
            ],
            [Empty, BottomRight, Empty, Empty, Empty, Empty, TopRight],
            [TopLeft, Empty, Empty, Empty, Empty, Empty, TopLeft],
            [TopRight, Empty, Empty, Empty, Empty, Empty, TopRight],
            [TopLeft, Empty, Empty, Empty, Empty, Empty, TopLeft],
            [TopRight, Empty, Empty, Empty, Empty, BottomRight, Empty],
            [
                Empty, BottomLeft, TopLeft, BottomLeft, TopLeft, Empty, Empty,
            ],
        ]);

        assert!(grid3.loop_area().unwrap().is_integer(32));
    }
}
