//! This crate contains two implementations of a backtracking search algorithm to enumerate closed
//! curves in a 7x7 grid, as per the Jane Stree Puzzle 'Arc Acreage' from April 2023 (<https://www.janestreet.com/puzzles/archive/>).
//!
//! The first implementation was slow but produced the correct answer (depending on the strength of
//! search constraints used, the result took on the order of one hour to return). A faster technique
//! dramatically reduces the search space and does not require proving that any special constraints
//! must hold for curves of our target area. This algorithm returns the result in <100ms.
//!
//! As required in the original puzzle, there are 89,519,144 closed curves of area 32.

pub mod fast;
pub mod slow;

fn main() {
    fast();
}

#[allow(dead_code)]
fn fast() {
    use fast::*;

    let target_area = Area { units: 32, half: 0 };
    let (valid_cnt, valid_grids) = Generator::new(target_area, 49, 49).generate();

    // Double check validity.
    for valid in &valid_grids {
        if !(valid.loop_area().expect("should be valid").simplify() == target_area.simplify()) {
            println!("{:?}", valid);
            println!("area: {:?}", valid.loop_area());
        }
    }

    println!();
    println!(
        "Found {} valid grids with target area {}",
        valid_cnt, target_area
    );
}

#[allow(dead_code)]
fn slow() {
    use slow::*;

    let target_area = Area {
        units: 32,
        small: 0,
        large: 0,
    };

    let valid_grids = Generator::new(target_area, 6, 26).generate();

    // Double check validity.
    for valid in &valid_grids {
        if !(valid.loop_area().expect("should be valid").simplify() == target_area.simplify()) {
            println!("{:?}", valid);
            println!("area: {:?}", valid.loop_area());
        }
    }

    println!();
    println!(
        "Found {} valid grids with target area {}",
        valid_grids.len(),
        target_area
    );
}
