# Robot Long Jump

The [March 2023 Jane Street puzzle](https://www.janestreet.com/puzzles/robot-long-jump-index/) was about finding the Nash equilibrium for an interesting probabilistic game, couched in terms of robots competing at a kind of long jump. It was a pretty tricky problem; at least, I certainly found it required quite a wide range of different ideas and a lot of fiddly algebra (which I found almost impossible without the help of a computer algebra system). I'd be very interested to know if others found a more direct and elegant path to the finish line. Nevertheless, since far fewer people seemed to get the solution as in other months, and the [Jane Street solution page](https://www.janestreet.com/puzzles/robot-long-jump-solution) is quite light on detail, I thought I'd write up my approach here.

## The Problem

A reminder of the problem:

> A pair of robots are competing in a head-to-head long jump contest.
>
> These contests consist of rounds in which each robot has a single attempt to score. In an attempt, a robot speeds down the running track (modeled as the numberline) from 0, the starting line, to 1, the takeoff point. A robot moves along this track by drawing a real number uniformly from $[0,1]$ and adding it to the robot’s current position. After each of these advances, the robot must decide whether to jump or wait. If a robot crosses the takeoff point (at 1) before jumping its attempt receives a score of 0. If the robot jumps before crossing 1, it draws one final real number from $[0,1]$ and adds it to its current position, and this final sum is the score of the attempt.
>
> In a head-to-head contest, the two robots each have a single attempt without knowing the other’s result. In the case that they tie (typically because they both scored 0), that round is discarded and a new round begins. As soon as one robot scores higher than the other on the same round, that robot is declared the winner!
>
> Assume both robots are programmed to optimize their probability of winning and are aware of each other’s strategies. You are just sitting down to watch a match’s very first attempt (of the first round, which may or may not end up being discarded). **What is the probability that this attempt scores 0?** Give this probability as a decimal **rounded to 9 digits past the decimal point.**

## Seabo's Solution

### The basic strategy

The first observation to make is that the only feasible 'strategy' we could employ is to nominate a fixed threshold value, let's call it $x \in [0, 1]$, such that we choose to keep running if we are still before the threshold $x$, and if we find ourselves past the threshold but still before 1, we choose to jump. Note that it's possible to leap from a point before $x$ straight past 1, without having the opportunity to choose to jump. As explained in the statement of the question, this results in a score of 0 for the attempt (it's supposed to be akin to a real long-jumper overstepping the foul line at the takeoff point). So the whole problem revolves around analysing the implications of this strategy.

### Optimal strategy playing solo

The first interesting thing to do is to work out the value of $x$ which maximises our expected landing point. We can shortcut some calculus by just considering that the optimal value of $x$ will have the property that jumping from that point gives the same expected landing point as making one final running step and _then_ jumping. This is effectively what we would be working out if we calculated the expected landing point as a function of all values of $x$, and then differentiated that function to find the optimum. The expected landing position if we jump immediately from $x$ is simply $x + \frac{1}{2}$, since our jump has distance drawn uniformly from $[0, 1]$. Alternatively, if we were to make a final running step and then jump, we would have probability $(1-x)$ of not overshooting. Assuming we don't overshoot, our takeoff point is expected to be $\frac{1+x}{2}$ and then we again add $+1/2$ for the expected distance of the jump itself. Setting these two alternatives equal to each we get the following equation:

$$
\begin{align}
&\left(\frac{1+x}{2}+\frac{1}{2}\right)(1-x) = x + \frac{1}{2} \\
&\implies x^2 +3x - 1 = 0 \\
&\implies x = \frac{\sqrt{13}-3}{2} \approx 0.30278.
\end{align}
$$

It's worth running some quick Python simulations to verify that this value really is optimal. I got a curve that looked like the below, which suggests it is.

![](https://i.imgur.com/jIT0NcJ.png)

It may be tempting to think that if 0.30278 is the optimal threshold value to maximise one's own long jump score, this must be the optimum for two robots competing head-to-head - especially given that they act completely independently and don't even have knowledge of the other's choices. However, this is _not_ actually the case!

In fact, this problem reminded me of another puzzle I once worked through which had a similar property. I think it's worth a quick diversion, because this seems to be a recurring theme and an important property to keep in mind when tackling these game theory set-ups.

### Diversion: another random number game

> Suppose you have a random number generator which produces numbers uniformly in $[0,1]$. You press the button and get a number out. You now have a choice of whether to stick with the number you see, or whether to press again. If you press again, you must stick with the second number.
>
> You are playing this game against an opponent who is playing the exact same setup with a different random number generator. Neither of you can see the other’s actions or the outcomes of their rolls. You each play the game _completely independently_ and whoever achieves the highest score wins.

The surprising thing about this game (as you may be guessing) is that the best strategy to win against an opponent is _not_ the same as the best strategy to maximise your own expected score, _even though_ you both act independently and there is no direct interaction of any kind. To maximise your own expected score is relatively obvious: you should roll again if your first roll is less than 0.5, because the expected value of your re-roll is 0.5. This gives you an expected score of 0.625. However, imagine you're playing against an opponent known to be using this strategy. What would you do? If you know they are going to behave like this, you can slightly increase your chances of winning by re-rolling whenever you score _less than 0.625_ (rather than 0.5). Since you know _their_ expected score is 0.625, you should not be content to settle for a score in the range $[0.5, 0.625]$ on the first roll, since you can expect this to lose against the opponent on average. So you might as well roll again. Using the new threshold of 0.625 yields a new expected value for attempt. And this similarly modifies your opponent's best strategy, because now they should use _your_ expected value as _their_ threshold for re-rolling! In fact, we need to iterate this forever until we reach a fixed point of the process. That fixed point is the _unexploitable strategy_ that two perfectly rational players would pick. I will leave it to the reader to run through the algebra (there's plenty of that on the way!), but the optimal threshold for this game turns out to be something associated with the golden ratio. Of course.

### Nash equilibrium

In the diversion, what we actually did was find the Nash equilibrium for the game. That's the set of strategies (i.e. re-rolling thresholds) where, even if all the players knew what thresholds everyone else was using, they wouldn't feel inclined to change their own strategy in an effort to outmaneouver their opponents and get a higher chance of winning. We need to do something similar for our long jump game.

First, you can convince yourself that this is required by running some more simulations in Python. If we have two independent players, and one of them uses the $\frac{\sqrt{13}-3}{2}$ threshold, can the other player find a different threshold that increases their probability of winning above 50%?

Here's a graph showing jumping thresholds for the second player on the x-axis and their probability of winning against the first player, who uses the 'solo-optimal' threshold of 0.30278.

![](https://i.imgur.com/bw39Rax.png)

It turns out that if the we use a threshold of 0.41, we get a 50.8% chance of winning! Slightly better than the 50.0% we would get if we also used the 'solo-optimal' threshold just like our opponent. So we are definitely in a similar situation to what we saw in the diversion, and must press ahead to find a Nash equilibrium for this game.

In terms of the diagram above, the Nash equilibrium will be a value for the threshold, call it $x^*$, such that if player 1 uses that threshold, $x^*$ would _also_ be the threshold which player 2 would use to maximise their probability of winning on the graph (i.e. the x-axis value where the maximum point of the graph is). And that probability would of course be back to 50.0% at that point, by symmetry.

### Overshooting

Before we finally get into the (gruelling) calculations I ran to arrive at a solution for $x^*$, we need one more preliminary result. We will need to know what the probability of overshooting is for any given threshold value $x$.

To get this actually requires a bit more machinery. For a given number $x$, what is the probability that the sum of $n$ independent, uniformly random numbers all drawn from $[0, 1]$ is less than $x$? The answer to this turns out to be $x^n/n!$. You can see a couple of derivations of that result [here](https://math.stackexchange.com/questions/1683558/probability-that-sum-of-independent-uniform-variables-is-less-than-1). There's a relatively well-known puzzle which asks what the expected number of uniform $[0, 1]$ values is required for the sum to exceed 1; the answer is $e$, which you should be able to get relatively easily from the above result.

Armed with this, we can work out the chance of overshooting. Overshooting happens when we take $n$ running steps without exceeding our threshold $x$, followed by a final running step which immediately exceeds 1, without landing in the range $[x, 1]$. We want to work out the probability of this happening for a given number of steps $n$, and then sum over all $n$ to get the overall probability. We get

$$
\sum_n \int_0^x \frac{t^n}{n!}t \space dt.
$$

This integrates over all the possible points $t$ in $[0, 1]$ which represent the location of our final running step before the final 'overshoot step'. The probability of having $n$ steps without exceeding $t$ is $t^n/n!$, as discussed above, and the probability of the next step jumping from $t$ into the region $[1, 1+t]$ is just $t$. We multiply these together (since the events are independent), integrate over all relevant $t$ and sum over all $n$. Using the well-known formula for $e^x$, this becomes:

$$
\int_0^x te^t \space dt = 1 - (1-x)e^x,
$$

which is our desired probability of an overshoot. As ever, we can easily verify this by running some simulations in Python.

### Path to the solution

To find the Nash equilibrium for the game, we're going to take the following steps:

- Work out the probability of winning if we use a threshold $y$ against an opponent using a different threshold $x$. This will be some function $P(x, y)$.
- Differentiate $P$ with respect to $y$, since we are seeking to optimise our probability of winning against a fixed value of $x$.
- Set $\partial P / \partial y = 0$, to give some new equation in terms of $x$ and $y$. In this equation, if we consider $x$ fixed, the value of $y$ which satisfies the equation will be the optimal value of $y$ which maximises the probability of winning against an opponent using $x$.
- In this equation, we will set $x = y$, representing the fixed-point Nash equilibrium. For our symmetrical game, a Nash equilibrium is defined by the property that $x$ is the optimum against $y$, and $y$ is the optimum against $x$, so they must be the same thing.
- This will now be an equation just in terms of $x$, and the solution to it will be $x^*$.
- Once we (finally) have our Nash equilibrium, we can simply plug it into the 'probability of overshoot' formula that we derived above, to get the final answer to the puzzle. Easy peasy.

So doing all of this begins with deriving the probability $P(x, y)$ of winning if we use threshold $y$ against an opponent using $x$.

For me, this was really quite fiddly, and as you'll see, my approach involved lots of integrals and careful consideration of ranges that various jumping off points and landing points can take. It also involves breaking things down into lots of cases. It's not pretty by any means, and I continue to wonder if there is something more elegant that I overlooked. If anyone knows of a better approach, let me know!

### Conditioning on no overshoot

For the next section, we're going work out the probability of winning with a threshold $y$ against an opponent using threshold $x$, _conditioned on the case where neither player overshoots_. Let's call this $p(x, y)$. This is the harder case to analyse, and we'll make an adjusting calculation afterwards to get the more general case, $P(x, y)$ which captures all combinations of overshoot and no-overshoot from both players.

One relevant observation to make is that, in cases where we don't overshoot, our jumping point lies in the range $[x, 1]$, and in fact it is uniformly distributed in this range because each step we take is uniformly distributed in $[0, 1]$, so conditioning on any given location for a penultimate running step in $[0, x]$, we will always get a uniform distribution in $[x, 1]$ for the final running step (which becomes our jumping point).

### Probability of winning

The first thing to do is to work out our probability of winning against a value $t$ representing the _landing_ location of our opponent. Let's call our opponent Player 1, and think of ourselves as Player 2; Player 1 uses threshold $x$ while we use threshold $y$. The expression for the probability we want will actually vary depending on where exactly $t$ is in the range $[0, 2]$; we'll have to work these out case-by-case. Once we have all of that, we can integrate over the possible outcomes of Player 1's attempt, using our probability of winning against that outcome as the integrand in each region.

As discussed above, for the rest of the analysis in this section we will make the assumption that neither player overshoots; we will correct for this in the next step.

So, to recap: we are using threshold $y$, and our opponent has already landed somewhere which we are denoting as $t$ (and since they did not overshoot, $t > 1$). We want to know our probability of winning (i.e. landing somewhere $>t$).

The cases are:

1. $t \in [0, y]$
2. $t \in [y, 1]$
3. $t \in [1, 1+y]$
4. $t \in [1+y, 2]$.

#### Case 1 ($t \in [0, y]$)

If $t < y$, then we are guaranteed to win. Our opponent has _landed_ at a position that is strictly before our takeoff point, so we will surely win (remember, all this is conditioned on nobody overshooting, so we don't have to consider that).

#### Case 2 ($t \in [y, 1]$)

We evaluate the following integral.

$$
\frac{1}{1-y} \left[ \int_y^t \int_t^{a+1} dc \space da + \int_t^1 da \right]
$$

The factor of $1/(1-y)$ at the front is due to the uniform distribution of our takeoff point in the range $[y, 1]$. We use the integration variable $a$ to represent our takeoff point in the range $[y, 1]$, and we split this integration range into the two outside integrals shown. If our takeoff point is greater than $t$ (the last integral), then we are guaranteed to win, hence the unit integrand. If our takeoff point is below $t$, then we will only win if our _landing point_ (denoted by variable $c$) is above $t$ (the upper limit of $a+1$ is the maximum we could attain for a jumping off point $a$).

Evaluating this expression gives

$$
\frac{(t-y)^2}{2(y-1)}+1.
$$

#### Case 3 ($t \in [1, 1+y]$)

We evaluate the following integral.

$$
\frac{1}{1-y} \int_y^1 \int_t^{a+1} dc \space da.
$$

This time, $t$ is somewhere between our own threshold $y$ and 1. We simply integrate over all our takeoff points $a$ and all the landing points in $[t, a+1]$ which would cause us to win.

Evaluating gives

$$
\frac{1}{2}(3 + y - 2t).
$$

#### Case 4 ($t \in [1+y, 2]$)

We evaluate the following integral.

$$
\frac{1}{1-y} \int_{t-1}^1 \int_t^{a+1} dc \space da.
$$

This time, $t$ is above 'our threshold plus 1'. It might help to draw a little diagram of the number line with all the relevant values marked on, but basically we can only beat this if our jumping off point is above $t-1$. If it was less than that, we would be more than 1 distance unit lower than our opponent landed, so we could never catch them up. The inner integral ranges over the values of our landing position $c$ which beat $t$.

Evaluating gives

$$
\frac{(2-t)^2}{2(1-y)}.
$$

So now we have a full picture of the probability of winning against any value of $t$ in the range $[0, 2]$, as a function of our threshold $y$. I find it can be useful to visualise these things, so here's a [Desmos graph](https://www.desmos.com/calculator/nvz7lf1bgm) of the function. The variables in that link are very confusingly renamed because you can't have variable names be whatever you want in Desmos as far as I know. The y-axis represents probability of winning; the x-axis is the opponent's landing point that we are trying to beat (called $t$ in the analysis above); the slider $d$ represent our takeoff point (called $y$ in the analysis above).

To summarise, for a given $t$ and a threshold $y$, the probability of beating $t$ given that we don't overshoot is given by:

$$
\mathbb{P}(\text{win vs. landing value } t \space | \space \text{no overshoot}) = \begin{cases}
1 & \text{if } \space 0 < t < y \\
\frac{(t-y)^2}{2(y-1)}+1 & \text{if } \space y < t < 1 \\
\frac{1}{2}(3 + y - 2t) & \text{if } \space 1 < t < 1+y \\
\frac{(2-t)^2}{2(1-y)} & \text{if } \space 1+y < t < 2.
\end{cases}
$$

#### Integrating over Player 1's landing values

Now that we know what our chance of winning is versus any given landing value $t$ that Player 1 may achieve, we need to integrate over all these possible landing values, using their probability density of occurring as well as our probability density of winning against that value of $t$. We need to split everything out into the ranges where the various formulas hold.

$$
\begin{aligned}
&p(x, y) = \mathbb{P}(\text{threshold } y \text{ wins vs. threshold } x \space | \space \text{no overshoot}) = \\
& \frac{1}{1-x} \int_x^y \left[ \int_a^y dt + \int_y^1 \frac{(t-y)^2}{2(y-1)} + 1 dt + \int_1^{a+1} \frac{1}{2}(3+y-2t) dt \right] da \\
&+ \frac{1}{1-x} \int_y^1 \left[ \int_a^1 \frac{(t-y)^2}{2(y-1)} + 1 dt + \int_1^{1+y} \frac{1}{2}(3+y-2t)dt + \int_{1+y}^{1+a} \frac{(2-t)^2}{2(1-y)} dt \right] da.
\end{aligned}
$$

That looks a bit monstrous but we can break it down relatively easily. Outside the big square brackets we have two integrals ranging over all the values $a$ can take, where $a$ denotes the jumping off point for Player 1. This range is $[x, 1]$ overall, but we have to split it into the two ranges $[x, y]$ and $[y, 1]$ in order to accurately apply the probability-of-winning formulas to the relevant regions. In particular, when $a \in [x, y]$, it is possible for Player 1's landing point to be _below_ $y$ (Player 2's threshold). This situation is represented by the first integral inside the first square brackets. In this region, Player 2 is guaranteed to win, hence the unit integrand. It's not too difficult to work through each of the integrals and think about which scenarios they represent. All of the integrands are segments from the function we worked out in the previous sub-section.

Note also that the factor of $1/(1-x)$ arises again from the fact that we are conditioning on neither player overshooting. When this happens, the jumping off point $a$ is uniformly located in the range $[x, 1]$, as discussed earlier - hence the constant factor out front.

Actually evaluating the expression above was too much like hard work for me, so I plugged it all through Wolfram Alpha and got the following out:

$$
p(x, y) = \frac{(2x^3 - 3x^2(y-1) + 2x(y^2-2y-5)-y^3 + y^2 + 4y +6)}{12(1-x)}. \quad (\star)
$$

So to recap, the expression above represents our probability of winning if we are using the threshold $y$ against an opponent using $x$, where $x < y$ and conditioned on neither of us overshooting. Phew.

#### Unconditioning

We need an overall expression for the probability of winning with threshold $y$ against threshold $x$ which _isn't_ conditioned on both players avoiding an overshoot. To do that, we need to reconsider how the game works. If one player overshoots and the other doesn't, the player who didn't overshoot is guaranteed to win, no matter what they scored. If neither player overshoots, we already have the answer above: $p(x, y)$. If both players overshoot, they scrap those attempts and start back from the beginning. We can capture all of this as follows, using $P(x, y)$ to denote the overall unconditional probability of winning with threshold $y$ versus threshold $x$ (where $x < y$).

$$
P(x, y) = (\star) \underbrace{(1-x)e^x(1-y)e^y}_{\text{neither player overshoots}} + \underbrace{(1-y)e^y (1 - (1-x)e^{x})}_{\text{Player 1 overshoots, Player 2 doesn't}} + \\ \underbrace{(1-(1-x)e^x)(1-(1-y)e^y))}_{\text{both players overshoot}}P(x, y).
$$

Here, we've used the overshoot probability formulas derive earlier. Namely:

- probability of overshooting with threshold $x$: $1 - (1-x)e^x$
- probability of not overshoot with threshold $x$: $(1-x)e^x$.

There are three terms in the formula for $P(x, y)$. The first represents a situation where neither player overshoots, and uses the big formula $(\star)$ from the last section. The next term represents a situation where Player 2 does not overshoot but Player 1 does (whereupon Player 2 definitely wins). The final term represents a situation where both players overshoot. In this case, the formula recursively references the probability $P(x, y)$, since the players scrap their attempts and start from scratch, meaning that Player 2 once again has overall probability $P(x, y)$ of winning. (Note that no term appears for the situation where Player 2 overshoots and Player 1 doesn't, because in that case Player 2 is guaranteed to lose and the term would be multiplied by a factor of 0.) We can rearrange this formula to get it in terms of $P$:

$$
P(x, y) = \frac{(\star)(1-x)(1-y)e^{x+y} + (1-y)e^y - (1-x)(1-y)e^{x+y}}{(1-x)e^x + (1-y)e^y - (1-x)(1-y)e^{x+y}}.
$$

### Crossing the finish line (/ takeoff point?)

The final few steps are heavy-duty rote algebra, so I used Wolfram Alpha once again to get it done. We need to work out $\partial p / \partial y$, set this to 0 to solve for the optimising value of $y$ against an opponent playing strategy $x$, and then set $y=x$ in the resulting equation. Because we are differentiating a quotient, we'll end up with a gnarly expression that has a squared factor on the denominator; so when setting equal to 0, we can throw away the denominator and focus solely on the numerator. Whichever values of $x$ satisfy this equation are candidates for our Nash equilibrium strategy $x^*$ --- at last!

Crunching through all of that algebra, we end up with:

$$
2\underbrace{(e^x(x-1)+2)}_{>0 \space \forall x} \underbrace{(x-1)}_{\text{no good}} \underbrace{(e^x(x-1)^2(x+2)-3x)}_{\text{has roots}} = 0.
$$

Of the three factors, the first has no roots, the second has a root at $x=1$ which cannot possibly be the Nash equilibrium since we would certainly overshoot on every attempt. So the only candidate to contribute a root is the final factor. Setting this to 0, we get

$$
(x^3-3x+2)e^x = 3x,
$$

which finally yields the cryptic formula referenced on the [Jane Street solution page](https://www.janestreet.com/puzzles/robot-long-jump-solution). Unfortunately, there's no nice closed form solution for this, so after all that work we are forced to evaluate $x^*$ numerically. You could do this by writing some kind of iterative numerical code (like Newton-Raphson); I stuck with Wolfram Alpha. We get out the value 0.416195355 as the only real root in the range $[0, 1]$. Finally, we can plug this value into our old formula for the probability of an overshoot to get the value **0.114845886**, as requested in the problem statement.

### Final thoughts

I still feel like this solution is seriously hard work and can't shake the feeling that there must be some more direct and elegant route to the answer. If there is, I wasn't able to spot so I'd be fascinated to hear from smarter people than me. Let me know!
