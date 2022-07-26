# Andy the Ant

This repository contains Rust code I wrote in an effort to solve the [Andy
the Ant](https://www.janestreet.com/puzzles/current-puzzle/) problem.

## The Puzzle

Andy the ant has spent most of his days living on a strange land consisting of white hexagons that are surrounded by alternating black pentagons and white hexagons (three of each), and black pentagons surrounded by five white hexagons. To us this land is familiar as the classic soccer ball we see above on the left. Due to Andy’s tiny size and terrible eyesight, he doesn’t notice the curvature of the land and avoids the black pentagons because he suspects they may be bottomless pits.

Every morning he wakes up on a white hexagon, leaves some pheromones to mark it has his special home space, and starts his random morning stroll. Every step on this stroll takes him to one of the three neighboring white hexagons with equal probability. He ends his stroll as soon has he first returns to his home space. As an example, on exactly 1/3 of mornings Andy’s stroll is 2 steps long, as he randomly visits one of the three neighbors, and then has a 1/3 probability of returning immediately to the home hexagon.

This morning, his soccer ball bounced through a kitchen with an infinite (at least practically speaking…) regular hexagonal floor tiling consisting of black and white hexagons, a small part of which is shown above on the right. In this tiling every white hexagon is surrounded by alternating black and white hexagons, and black hexagons are surrounded by six white hexagons. Andy fell off the ball and woke up on a white hexagon. He didn’t notice any change in his surroundings, and goes about his normal morning routine.

Let $p$ be the probability that his morning stroll on this new land is strictly more steps than the expected number of steps his strolls on the soccer ball took. Find $p$, rounded to seven significant digits.

## Solution

The code for the full solution is in this repository. The [crate
documentation](https://js.seabo.me) contains detailed explanations of
the methodology and implementation.




