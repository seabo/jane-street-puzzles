# Arc Acreage

This repository contains Rust code I wrote to solve the [Arc Acreage](https://www.janestreet.com/puzzles/current-puzzle/)
problem.

## The Puzzle

![Arc acreage. A 7x7 grid with closed curves built from quarter-circle
arc segments](https://www.janestreet.com/puzzles/arc-edge-acreage.png)

In the 7-by-7 grid above, one can draw a simple closed curve using
nothing but quarter-circle segments. Two examples are shown above: one
enclosing a region of 4-π (in blue), and one enclosing a region of area
6 (in red).

It can be shown that there are 36 ways to enclose a region of area
exactly 4-π. How many ways can one draw a curve enclosing a region of
area exactly 32?

Note that a simple curve is not allowed to self-intersect.

## Solution

The code for the full solution is in this repository. The [crate
documentation](https://js.seabo.me/arc_acreage) contains some further
details and explanations of the methodology.
