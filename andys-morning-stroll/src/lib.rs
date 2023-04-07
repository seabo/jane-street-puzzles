#![feature(associated_type_defaults)]
#![allow(dead_code)]
#![deny(missing_docs)]
//! Algorithms used to solve [Andy's Morning Stroll](https://www.janestreet.com/puzzles/current-puzzle/):
//!
//! > Andy the ant has spent most of his days living on a strange land consisting of white hexagons
//! > that are surrounded by alternating black pentagons and white hexagons (three of each), and
//! > black pentagons surrounded by five white hexagons. To us this land is familiar as the classic
//! > soccer ball we see above on the left. Due to Andy’s tiny size and terrible eyesight, he doesn’t
//! > notice the curvature of the land and avoids the black pentagons because he suspects they may be
//! > bottomless pits.
//! >  
//! > Every morning he wakes up on a white hexagon, leaves some pheromones to mark it has his special
//! > home space, and starts his random morning stroll. Every step on this stroll takes him to one of
//! > the three neighboring white hexagons with equal probability. He ends his stroll as soon has he
//! > first returns to his home space. As an example, on exactly 1/3 of mornings Andy’s stroll is 2
//! > steps long, as he randomly visits one of the three neighbors, and then has a 1/3 probability
//! > of returning immediately to the home hexagon.
//! >
//! > This morning, his soccer ball bounced through a kitchen with an infinite (at least practically
//! > speaking…) regular hexagonal floor tiling consisting of black and white hexagons, a small part
//! > of which is shown above on the right. In this tiling every white hexagon is surrounded by
//! > alternating black and white hexagons, and black hexagons are surrounded by six white hexagons.
//! > Andy fell off the ball and woke up on a white hexagon. He didn’t notice any change in his
//! > surroundings, and goes about his normal morning routine.
//! >  
//! > Let $p$ be the probability that his morning stroll on this new land is strictly more steps than
//! > the expected number of steps his strolls on the soccer ball took. Find $p$, rounded to seven
//! > significant digits.
//!
//! # First part
//!
//! Initially, I wrote the [RandomWalk](crate::RandomWalk) trait and the [Football](crate::Football)
//! struct to implement the walk on a football. The algorithm seemed to strongly hint that the answer
//! was 20, which interestingly is the number of nodes in the graph.
//!
//! It's actually not difficult to prove this by taking advantage of the symmetry of the graph and
//! down some relations between the various expected values. Let $E_k$ be the expected length of a
//! starting at node $k$ and walking until we first reach node $0$. Then we wish to find $E_0$.
//! Drawing out the graph, we can see that, by symmetry, there are only 5 distinct types of nodes,
//! where each type must have the same value of $E_k$, so let's just use $k$ to denote the node,
//! rather than the specific node (i.e. the equivalance class). Then it's easy to see that the
//! relationships between the $E_k$ are:
//!
//! * $E_0 = E_1 + 1$
//! * $3E_1 = 3 + 2E_2$
//! * $3E_2 = E_1 + E_2 + E_3 + 3$
//! * $3E_3 = E_2 + E_3 + E_4 + 3$
//! * $3E_4 = 2E_3 + E_5 + 3$
//! * $E_5 = E_4 + 1$
//!
//! Starting with $E_4$ we can progressively substitute out the next highest $E_k$ from each
//! equation to ultimately solve for $E_0 = 20$.
//!
//! # Second part
//!
//! Having solved the first half I then created a new struct [KitchenFloor](crate::KitchenFloor)
//! which implements the [RandomWalk](crate::RandomWalk) trait, to stochastically measure how
//! frequently Andy has a walk that is longer than 20 steps. This approach gives an answer of
//! roughly 0.448, but it wasn't converging quickly event with a large number of runs, so even after
//! implementing a multithreaded version to get the total number of runs up to 8 billion, I still
//! needed a different approach.
//!
//! I came up two, both of which are implemented below:
//!
//! 1. Enumerate every possible walk from the possible $3^{20}$ available (3 choices at each
//!    step), then iterate through them (approx. 3.5 billion), counting which ones terminate by
//!    arriving back at the home hexagon on or before the 20th step. This is not too bad
//!    computationally, but 3.5 billion possibilities take several minutes to calculate on a fast
//!    machine.
//! 2. Build the graph in-memory, and iterate through the 20 steps, tracking how many paths have
//!    arrived at each node on the $n$th step by summing surrounding nodes from the previous step.
//!    This is certainly the smartest and most computationally efficient solution, yielding a
//!    solution in c.1 millisecond on the same machine.
//!
//! Both approaches above gave the same answer, which also matched the stochastic estimate of
//! 0.448. So the solution is
//!
//! $$p = 0.4480326 \text{ (7 s.f.)}$$

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use std::collections::HashMap;

/// Implement random walks on a state machine.
pub trait RandomWalk {
    /// The representation of state in this state machine.
    type State: PartialEq + Clone;

    /// Make a random move on the internal state machine.
    fn make_move<R: Rng>(&mut self, rng: &mut R);

    /// Return the current state of the machine.
    fn get_state(&self) -> Self::State;

    /// Set the state of the internal state machine.
    fn set_state(&mut self, state: Self::State);

    /// Perform a random walk, starting at `src`, and making random moves until the `tgt` state is
    /// reached. This does not terminate at zero steps if `src` and `tgt` are the same, a move is
    /// always made first before continuing until `tgt`.
    ///
    /// Returns the number of steps it took. This method could block forever if the state
    /// diverges somehow and never arrives at `tgt`. See also `walk_until_limit`.
    fn walk<R: Rng>(&mut self, src: Self::State, tgt: Self::State, rng: &mut R) -> u32 {
        self.set_state(src);

        self.make_move(rng);
        let mut cnt = 1u32;

        while self.get_state() != tgt {
            self.make_move(rng);
            cnt += 1;
        }
        cnt
    }

    /// Same as `walk_until`, but also takes a `limit` parameter, specifying the maximum length of
    /// the walk we should allow before bailing out. Returns `Ok(num_steps)` if `tgt` is reached at or
    /// before the limit, and `Err(limit)` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is zero.
    fn walk_until_limit<R: Rng>(
        &mut self,
        src: Self::State,
        tgt: Self::State,
        rng: &mut R,
        limit: u32,
    ) -> Result<u32, u32> {
        if limit == 0 {
            panic!("limit should be > 0");
        }
        self.set_state(src);

        self.make_move(rng);
        let mut cnt = 1u32;

        while self.get_state() != tgt && cnt < limit {
            self.make_move(rng);
            cnt += 1;
        }

        if cnt == limit {
            if self.get_state() == tgt {
                Ok(limit)
            } else {
                Err(limit)
            }
        } else {
            Ok(cnt)
        }
    }
}

/// A representation of the football Andy the Ant lives on.
///
/// A football is a [truncated icosahedron](https://en.wikipedia.org/wiki/Truncated_icosahedron).
/// Andy lives on the hexagons, of which there are 20. The pentagon-centered stereographic projection
/// from the Wikipedia page is useful to look at to see the graph structure of Andy's universe (he
/// lives on the yellow faces in that diagram).
///
/// In order to model his universe, we can simply use integers from 0 to 20 for each possible face,
/// and define the available transitions manually.
pub struct Football {
    curr: i32,
    transitions: HashMap<i32, [i32; 3]>,
    dist: Uniform<usize>,
}

impl RandomWalk for Football {
    type State = i32;

    fn make_move<R: Rng>(&mut self, rng: &mut R) {
        let possibles = self.transitions.get(&self.curr).unwrap();
        let random_idx = self.dist.sample(rng);
        self.curr = *possibles.get(random_idx).unwrap();
    }

    fn get_state(&self) -> Self::State {
        self.curr
    }

    fn set_state(&mut self, state: Self::State) {
        self.curr = state;
    }
}

impl Football {
    /// Create a football.
    pub fn new() -> Self {
        // Generated by randomly labelling the hexagons on the stereographic projection and manually
        // hardcoding the transition matrix. It would be interesting to think about ways to
        // programmatically generate things like this, but for now, this is quicker.
        let transitions: HashMap<i32, [i32; 3]> = HashMap::from([
            (1, [2, 6, 5]),
            (2, [1, 7, 3]),
            (3, [4, 8, 2]),
            (4, [3, 9, 5]),
            (5, [4, 10, 1]),
            (6, [1, 11, 12]),
            (7, [2, 12, 13]),
            (8, [3, 13, 14]),
            (9, [4, 14, 15]),
            (10, [5, 11, 15]),
            (11, [6, 10, 20]),
            (12, [6, 7, 16]),
            (13, [7, 8, 17]),
            (14, [8, 9, 18]),
            (15, [9, 10, 19]),
            (16, [12, 17, 20]),
            (17, [13, 16, 18]),
            (18, [14, 17, 19]),
            (19, [15, 18, 20]),
            (20, [11, 16, 19]),
        ]);

        Self {
            transitions,
            curr: 1,
            dist: Uniform::from(0..3),
        }
    }
}

/// An implementation of the infinite hexagonally tiled kitchen floor Andy unwittingly found
/// himself walking around on this morning.
///
/// To implement this, we can use a coordinate system (x, y), where each hexagon, including the
/// black ones, has a coordinate assigned.
///
/// There are two distinct types of white hexagon, which we can characterise by the compass
/// directions we can move to adjacent white hexagons:
///  A. (NW, SW, E)
///  B. (W, NE, SE)
///
///  We can map the co-ordinates of any given white hexagon to ascertain its type by:
///  ```
///  hex_type: bool = (y % 3) == (3 - x) % 3
///  ```
///
///  The available moves in each case are:
///  * `hex_type == 0`: `(x, y) -> [(x+1, y+1), (x, y-1), (x-1, y)]`
///  * `hex_type == 1`: `(x, y) -> [(x, y+1), (x-1, y-1), (x+1, y)]`
pub struct KitchenFloor {
    coords: (i32, i32),
}

impl KitchenFloor {
    /// Create a new kitchen floor.
    fn new() -> Self {
        Self { coords: (0, 0) }
    }

    fn coord_hex_type(coord: (i32, i32)) -> bool {
        let x = coord.0;
        let y = coord.1;
        y.rem_euclid(3) == (3 - x).rem_euclid(3)
    }

    fn coord_neighbours(coord: (i32, i32)) -> [(i32, i32); 3] {
        if Self::coord_hex_type(coord) {
            [
                (coord.0 + 1, coord.1 + 1),
                (coord.0, coord.1 - 1),
                (coord.0 - 1, coord.1),
            ]
        } else {
            [
                (coord.0, coord.1 + 1),
                (coord.0 + 1, coord.1),
                (coord.0 - 1, coord.1 - 1),
            ]
        }
    }

    /// For simplicity, we use a boolean to encode the two types of hexagon we could be on.
    fn hex_type(&self) -> bool {
        Self::coord_hex_type(self.coords)
    }
    fn neighbours(&self) -> [(i32, i32); 3] {
        Self::coord_neighbours(self.coords)
    }

    fn move_from_idx(&mut self, idx: usize) {
        if self.hex_type() {
            match idx {
                0 => {
                    self.coords.0 += 1;
                    self.coords.1 += 1
                }
                1 => self.coords.1 -= 1,
                2 => self.coords.0 -= 1,
                _ => unreachable!(),
            };
        } else {
            match idx {
                0 => self.coords.1 += 1,
                1 => {
                    self.coords.0 -= 1;
                    self.coords.1 -= 1;
                }
                2 => self.coords.0 += 1,
                _ => unreachable!(),
            }
        }
    }
}

impl RandomWalk for KitchenFloor {
    type State = (i32, i32);

    fn make_move<R: Rng>(&mut self, _rng: &mut R) {
        let random_idx = fastrand::usize(..3);
        self.move_from_idx(random_idx);
    }

    fn get_state(&self) -> Self::State {
        self.coords
    }

    fn set_state(&mut self, state: Self::State) {
        self.coords = state;
    }
}

/// A struct to calculate the expected length of a random walk, for any type `T: RandomWalk`. We
/// will use this to calculate the expected values of walks on our [Football](crate::Football) and
/// [KitchenFloor](crate::KitchenFloor) types.
pub struct Expectation<T: RandomWalk> {
    /// The model of our random walk.
    walker: T,

    /// A map from walk lengths to the frequency of occurences of walks of that length.
    pub freq_map: HashMap<u32, u32>,

    /// Count of the number of runs executed so far.
    pub cnt: u32,
}

impl<T: RandomWalk> Expectation<T> {
    /// Create a new expectation calculator.
    pub fn new(walker: T) -> Self {
        Self {
            walker,
            freq_map: HashMap::new(),
            cnt: 0,
        }
    }

    /// Run the expectation computation. Internally, this calls `walker.walk` which does not take a
    /// limit cut-off for walk lengths. Therefore, this function could take a long time if walks
    /// can be extremely long or even diverege to infinity.
    pub fn calculate(&mut self, src: T::State, tgt: T::State, runs: u32) -> f32 {
        let mut rng = rand::thread_rng();
        while self.cnt < runs {
            let steps = self.walker.walk(src.clone(), tgt.clone(), &mut rng);
            *self.freq_map.entry(steps).or_insert(0) += 1;
            self.cnt += 1;
        }

        self.finish()
    }

    /// Same as [calculate](Expectation::calculate) but takes a `limit` argument which is passed to
    /// [RandomWalk::walk_until_limit](RandomWalk::walk_until_limit) in order to ensure the function
    /// terminates, ideally in a reasonable time.
    pub fn calculate_with_limit(
        &mut self,
        src: T::State,
        tgt: T::State,
        runs: u32,
        limit: u32,
    ) -> f32 {
        let mut rng = rand::thread_rng();
        while self.cnt < runs {
            let steps =
                match self
                    .walker
                    .walk_until_limit(src.clone(), tgt.clone(), &mut rng, limit)
                {
                    Ok(t) => t,
                    Err(t) => t,
                };
            *self.freq_map.entry(steps).or_insert(0) += 1;
            self.cnt += 1;
        }

        self.finish()
    }

    fn finish(&self) -> f32 {
        self.freq_map
            .iter()
            .fold(0, |acc, (walk_length, frequency)| {
                acc + walk_length * frequency
            }) as f32
            / self.cnt as f32
    }
}

/// Run this to get the answer to the first part of the question.
///
/// Interestingly, switching to [calculate_with_limit](Expectation::calculate_with_limit) and running
/// this with a walk-length limit even as high as 80 or 90 seems to
/// give the wrong picture. The expected value appears to vary significantly even as we
/// continue increasing the walk length limit to high values, so very long walks are
/// contributing to the result (it is a fat-tailed distribution). This is corroborated when we print
/// out the frequency map and see that we are getting plenty of walks which are several hundred steps
/// long.
///
/// Using the [calculate](Expectation::calculate) method which imposes no limitation on walk-lengths, but takes longer
/// to run, we seem to immediately be converging on an expected walk-length of 20 (which is of
/// course the total number of nodes in the graph). This is an interesting result, and one we later
/// proved rigorously (see [crate-level documentation](./index.html#first-part)).
pub fn expected_walk_length_on_football() {
    let football = Football::new();
    let mut exp = Expectation::new(football);
    let runs = 100_000_000;
    let exp_walk_length = exp.calculate(1, 1, runs);
    println!("E(length of walk to return home): {}", exp_walk_length);
    println!("cnt: {}", exp.cnt);
    for (k, v) in &exp.freq_map {
        println!("{}: {}", k, v);
    }
}

/// Find the probability that the random walk on the kitchen floor is strictly more than 20 steps.
///
/// This is the first attempt to solve the second part of the question. We discovered that 20 steps
/// is the expected length of a walk on the football, so we want want to know what proportion of
/// walks on the kitchen floor last longer than 20 steps.
///
/// We will perform a large number of runs, counting how many reach 21 steps without
/// terminating.
///
/// With increasing runs, we seem to be converging towards about 0.448.
pub fn prob_of_longer_walk_in_the_kitchen() -> (u64, u64) {
    let mut kitchen_floor = KitchenFloor::new();
    let mut rng = rand::thread_rng();
    let runs: u64 = 10_000_000;
    let progress_unit = runs / 20;
    let mut progress_cnt = 0;
    let mut progress = 0;
    let mut cnt: u64 = 0;
    let mut longer_walk_cnt: u64 = 0;
    while cnt < runs {
        match kitchen_floor.walk_until_limit((0, 0), (0, 0), &mut rng, 20) {
            Ok(_) => {
                // We terminated on or before the 20th step. So this does not contribute to our count
                // of longer walks.
            }
            Err(_) => {
                // We had not terminated by the 20th step, so this does contribute to our count of
                // longer walks.
                longer_walk_cnt += 1;
            }
        };
        cnt += 1;
        progress_cnt += 1;
        if progress_cnt == progress_unit {
            progress += 1;
            println!(
                "{:2}% complete, {} runs, current prob: {}",
                progress as f32 * 5 as f32 as f32,
                cnt,
                longer_walk_cnt as f64 / cnt as f64
            );
            progress_cnt = 0;
        }
    }
    println!("runs longer than 20: {}", longer_walk_cnt);
    println!("total runs: {}", runs);
    println!(
        "probability of a longer than 20 walk: {}",
        longer_walk_cnt as f64 / runs as f64
    );
    (longer_walk_cnt, runs)
}

/// Multithreaded version of [prob_of_longer_walk_in_the_kitchen](prob_of_longer_walk_in_the_kitchen).
///
/// I've got a computer with lots of cpus, and running a monte carlo with indpendent trials is
/// silly to do single-threaded. So we can split this across a bunch of threads to do more
/// trials.
///
/// Running this with about 1 billion iterations per threads over 8 threads, we get to about an
/// estimate of our probability that the random walk is longer than 20 steps of: ~0.448
pub fn multithreaded() {
    let cpus = 8;
    let idx = 0..cpus;
    let mut join_handles: Vec<std::thread::JoinHandle<(u64, u64)>> = Vec::with_capacity(cpus);
    for _ in idx {
        join_handles.push(std::thread::spawn(|| prob_of_longer_walk_in_the_kitchen()));
    }
    let mut results: Vec<(u64, u64)> = Vec::with_capacity(cpus);
    join_handles
        .into_iter()
        .for_each(|jh| results.push(jh.join().unwrap()));

    let grand_total: (u64, u64) = results
        .iter()
        .fold((0, 0), |acc, e| (acc.0 + e.0, acc.1 + e.1));
    println!("grand total: {:?}", grand_total);
    println!(
        "probability of a longer than 20 walk: {}",
        grand_total.0 as f64 / grand_total.1 as f64
    );
}

/// A new approach to part 2. Enumerating every possible walk.
///
/// Our montecarlo approach doesn't seem to be converging on the correct answer fast enough for us
/// to believe in the 7 decimal places we need to get this right. However, we are only looking for
/// the number of terminating walks of less than 20 steps. We have a choice of three directions at
/// each step, so perhaps we can just enumerate every one of the $2^{30}$ = c. 3.5 billion possible
/// paths and get a precise answer.
///
/// Running this function (it's only single threaded so takes a few minutes - it could be made faster by
/// splitting the enumerable range across a few CPUs, as we have done elsewhere) we get:
///
/// $$
/// P(\text{random walk is longer than 20 steps}) = 0.4480326 \text{ (7 s.f.)}
/// $$
///
/// This figure closely matches the answer we were getting stochastically, so presume we have got
/// everything right.
pub fn enumerate_every_walk() {
    let max: usize = 3_usize.pow(20) - 1;
    let progress_unit = max / 20;
    let mut progress_cnt = 0;
    let mut progress = 0;

    // Count the walks which terminate in our first 20 steps.
    let mut terminated_cnt: usize = 0;

    let mut kitchen_floor = KitchenFloor::new();
    let mut decisions = Decisions::new();

    for i in 0..max {
        decisions.inc();
        kitchen_floor.set_state((0, 0));

        for dec in decisions.curr() {
            kitchen_floor.move_from_idx(*dec);
            if kitchen_floor.get_state() == (0, 0) {
                terminated_cnt += 1;
                break;
            }
        }

        progress_cnt += 1;

        if progress_cnt == progress_unit {
            progress += 1;
            println!(
                "{:2}% complete, {} runs",
                progress as f32 * 5 as f32 as f32,
                i,
            );
            progress_cnt = 0;
        }
    }

    println!("terminated walks (<= 20 steps): {}", terminated_cnt);
    println!(
        "non-terminated walks (> 20 steps): {}",
        max - terminated_cnt
    );
    println!(
        "probability > 20: {:10}",
        (max as f64 - terminated_cnt as f64) / max as f64
    );
}

/// A helper struct to assist with iterating through the possible choices of path.
///
/// We are essentially counting in base 3. For simplicity, we're using an array of 20 `usize`
/// integers.
///
/// To be cleaner, this should probably implement `Iterator` from the standard library.
pub struct Decisions {
    curr: [usize; 20],
}

impl Decisions {
    /// Create a new decision iterator.
    pub fn new() -> Self {
        Self { curr: [0; 20] }
    }

    /// Return the current value of the array representation of our path.
    pub fn curr(&self) -> &[usize; 20] {
        &self.curr
    }

    /// Increment to the next path.
    pub fn inc(&mut self) {
        for idx in 0..20 {
            if self.curr[idx] == 2 {
                self.curr[idx] = 0;
            } else {
                self.curr[idx] += 1;
                // As soon as we increment something, rather than reseting, we bail.
                return;
            }
        }
    }
}

/// A representation of a coordinate on our [KitchenFloor](KitchenFloor) plane.
pub type Coord = (i32, i32);

/// Stores a representation of the underlying graph, tracking how many paths have reached each node
/// at current time step `self.step`.
///
/// Calling [next](Self::next) steps the [KitchenFloor](KitchenFloor) steps the graph
/// representation forward by:
/// a) introducing any new nodes to the graph which haven't don't already exist from previous steps
/// b) iterating every node in the graph and setting its new value to the be the sum of the values
///    in the surrounding nodes from the previous step (but not counting any contribution from the
///    origin, because any paths which reached this on the previous step would have terminated
///    there).
pub struct GraphPathCounter {
    /// The kitchen floor model.
    kf: KitchenFloor,

    /// Tracks the total number of paths which can arrive at a given coord by a certain time step.
    pub cells: std::cell::RefCell<HashMap<Coord, usize>>,

    /// Tracks which time step we are currently at.
    step: usize,
}

impl GraphPathCounter {
    /// Create a new graph counter.
    pub fn new() -> Self {
        let counter = Self {
            kf: KitchenFloor::new(),
            cells: std::cell::RefCell::new(HashMap::new()),
            step: 0,
        };

        counter.cells.borrow_mut().entry((0, 0)).or_insert(1);
        counter
    }

    /// Step the internal graph representation forward.
    ///
    /// In summary, this function does works by:
    /// a) introducing any new nodes to the graph which haven't don't already exist from previous steps
    /// b) iterating every node in the graph and setting its new value to the be the sum of the values
    ///    in the surrounding nodes from the previous step (but not counting any contribution from the
    ///    origin, because any paths which reached this on the previous step would have terminated
    ///    there).
    pub fn next(&mut self) {
        self.step += 1;

        let cells: Vec<Coord> = self.cells.borrow().iter().map(|(c, _)| c.clone()).collect();
        for cell in cells {
            let neighbours = KitchenFloor::coord_neighbours(cell);

            for n in neighbours.iter() {
                // Ensure that the neighbour actually has an entry in the table.
                let _cell = self.cells.borrow_mut().entry(*n).or_insert(0);
            }
        }

        let mut new_values = Vec::new();

        // Now, we iterate over everything that appears in the table so far, and add to the counts
        // of each cell the sum of the counts of its neighbouring cells.
        for (cell, _) in self.cells.borrow().iter() {
            let mut new_cnt = 0;
            let cell_neighbours = KitchenFloor::coord_neighbours(*cell);

            for n in cell_neighbours.iter() {
                if *n != (0, 0) || self.step == 1 {
                    match self.cells.borrow().get(n) {
                        Some(n_cnt) => new_cnt += *n_cnt,
                        None => {}
                    };
                }
            }

            new_values.push((*cell, new_cnt));
        }

        for (cell, cnt) in new_values {
            *self.cells.borrow_mut().entry(cell).or_insert(0) = cnt;
        }
    }

    /// Run the analysis for a given number of steps.
    ///
    /// This function tracks how many paths return to the origin (0, 0) in total across all the
    /// steps. We need to be careful to upscale each such number by a factor of $3^k$ where $k$ is
    /// the number of remaining steps. This accounts for the fact that we stop counting the paths
    /// once they have returned home. If we kept counting them each one would diverge into
    /// $3^k$ paths over the remaining $k$ steps. We need to apportion the probability mass
    /// correctly in order to divide by $3^{20}$ total paths at the end.
    pub fn calculate(&mut self, steps: u32) {
        let start = std::time::Instant::now();
        let mut returning_paths = 0;
        let mut returned_paths = 0;
        for i in 0..steps {
            self.next();
            let returned_paths_at_step = self.cells.borrow().get(&(0, 0)).unwrap().clone();

            returned_paths += returned_paths_at_step * 3_usize.pow(steps - i - 1);
            returning_paths += returned_paths_at_step;
        }
        let returning_paths_at_final_step = self.cells.borrow().get(&(0, 0)).unwrap().clone();
        println!(
            "Number of returning paths on the {}th step: {}",
            steps, returning_paths_at_final_step
        );
        println!(
            "Number of returning paths within {} steps: {:?}",
            steps, returning_paths
        );

        let total_paths_at_final_step = self
            .cells
            .borrow()
            .iter()
            .fold(0, |acc, (_, cnt)| acc + cnt);
        println!("Total paths at final step: {}", total_paths_at_final_step);

        let total_paths =
            total_paths_at_final_step + returning_paths - self.cells.borrow().get(&(0, 0)).unwrap();
        println!("Total paths: {}", total_paths);

        println!(
            "Total paths inc. {}",
            returned_paths + total_paths_at_final_step - returning_paths_at_final_step
        );
        let max_paths = 3_usize.pow(steps);
        println!("3^{}: {}", steps, max_paths);

        println!(
            "p = {} / {} = {:7}",
            max_paths - returned_paths,
            max_paths,
            (max_paths - returned_paths) as f64 / max_paths as f64
        );
        println!("took {}ms", start.elapsed().as_micros());
    }
}

/// The most efficient way to calculate the solution to the second part of the question.
///
/// Running this we retrieve the final answer of $p = 0.4480326$ in c. 1ms.
pub fn path_counting_on_graph() {
    let mut counter = GraphPathCounter::new();
    counter.calculate(20);
}

#[cfg(test)]
mod tests {
    use crate::KitchenFloor;

    #[test]
    fn kitchen_floor_traversal() {
        #[rustfmt::skip]
    assert_eq!(KitchenFloor::coord_neighbours((0, 0)), [(1, 1), (0, -1), (-1, 0)]);

        #[rustfmt::skip]
    assert_eq!(KitchenFloor::coord_neighbours((-1, 1)), [(0, 2), (-1, 0), (-2, 1)]);

        #[rustfmt::skip]
    assert_eq!(KitchenFloor::coord_neighbours((-1, 0)), [(-1, 1), (0, 0), (-2, -1)]);

        #[rustfmt::skip]
    assert_eq!(KitchenFloor::coord_neighbours((-2, -1)), [(-1, 0), (-2, -2), (-3, -1)]);
    }
}
