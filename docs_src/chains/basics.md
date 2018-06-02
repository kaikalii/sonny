# Chain Basics

We have actually already been using chains. Let's look at the very first program we made that generates a tone.

```
std gen

output: A4 -> gen::sine -> out: 5
```

The word before the semicolon denotes the name of the chain, and the `->` operator separates links in the chain. The `out` keyword is special and is not actually considered a link. `gen::sine` is actually a chain itself, but we use the chain's name as an alias for the chain itself.

To learn to construct our own chains, let's implement our own version of `gen::sine`. First, we have to know how a sine wave is generated. To generate a sine wave with some frequency, we use the equation:

s<sub>t</sub> = sin(2 * π * t * f)

In this equation, **s<sub>t</sub>** is the amplitude of the sample at time **t**, **t** is the time, and **f** is the frequency of the wave.

We can make a very simple Sonny program that uses this equation to generate a 440 Hz tone.

```
my_sine: sin(2 * pi * time * 440) -> out: 5
```

There are a few things to note here. `sin` is a built-in unary operator in Sonny, and has the same precedence `-`(negate). `pi` and `time` are built-in constants. As the Sonny compiler iterates through every sample in the audio buffer, `time` evaluates to the time at that sample, which is based on the sample rate being used.

The program above will generate a 440 Hz tone just fine. However, it's not very reusable. Anyone using the my_sine chain would have to know exactly what number to change if they want to change the frequency.

If we look again at the first example program, we can see that the `gen::sine` link is preceded by the `A4` frequency. `gen::sine` uses this number when calculating the wave. To let out equation link reference the values of previous links, we use something called a **backlink**. Backlinks are denoted by a `!` followed by a positive integer. The backlink `!n` refers to the link that is that is n links before the link in which the backlink is.

To make our sine chain more generic and reusable, we can replace the `440` frequency number with the a backlink reference to the previous link.

```
my_sine: sin(2 * pi * time * !1) -> out: 5
```

Now, the link with the generator equation knows to use the result of the link that is fed into it. Because of this, attempting to compile this chain will result in an error.

```
TODO: put the error message here
```

We can fix this by actually feeding the link a frequency value.

```
my_sine: A4 -> sin(2 * pi * time * !1) -> out: 5
```

We are almost done. Because `my_sine` ends in `out`, no other chain will be able to use it. We can define `my_sine` ealier in the program so that it can be referenced in the output.

```
my_sine: sin(2 * pi * time * !1)

output: A4 -> my_sine -> out: 5
```

This program will compile and will generate our nice 5 second A4 tone. Hooray!

Backlinks are to Sonny what function arguments are to other languages. However, they have an interesting property that makes them unique. In most programming languages, if a function `f` takes a single argument can calls function `g` which takes three arguments, `f` still takes only one argument. In Sonny, the chains called from within a chain *can change the number of arguments of the caller*. Let's look at an example of this.

```
f: !1 + 1
g: !1 + !2 + !3
```

Here `f` is a simple chain that looks at the value passed to it from the previous link and adds `1` to it. `g` is a chain that adds the values passed to it by the three previous links.

As it is, `f` only requires a single link before it. In the anonymous chain below, `f` will only look at `4` and ignore `1` and `2`. It will return `5`.

```
1 -> 2 -> 4 -> f
```

However, if we change f to call `g`...

```
g: !1 + !2 + !3
f: !1 + g
```

`f` will now look at the values passed to it from the three previous links. Note that we had to change the order and define `g` first so that `f` can know about it. Our example chain:

```
1 -> 2 -> 3 -> f
```
will now now calculate `4 + 4 + 2 + 1 = 11`. If we substitute `g`'s body for its call in `f`, we can create an alternate version of `f` that does not call `g` at all. In the following snippet, chains `f1` and `f2` are equivalent.

```
g: !1 + !2 + !3
f1: !1 + g
f2: !1 + !1 + !2 + !3
```

Keep in mind that substituting a chain's body for its call only works if the chain has only a single link, like `g` does above.

How much a called chain extends the lookback of its caller depends on form which link in the caller it is called. Consider the following chains.

```
a: !1 + !2 + !3
b: !1 -> !1 -> !1
```

`a` adds the three values passed by the three previous links. Each link in `b` simply forwards the value of the previous link, so all `b` does is forward a value from input to output. The following chain evaluates to `6`.

```
1 -> 2 -> 3 -> a -> b
```

Let's make `b` call `a` in different places and see what happens.

```
a: !1 + !2 + !3
b: a -> !1 -> !1
c: 1 -> 2 -> 3 -> b
```

In the program above, `a` is called from the first link in `b`, so `b` now looks at the values passed to it from the three previous links in 'c'. `c` evaluates to `6`.

```
a: !1 + !2 + !3
b: !1 -> a -> !1
c: 1 -> 2 -> 3 -> b
```

In the program above, `a` is called from the second link in `b`, so `b` now looks at the values passed to it from the two previous links in 'c'. `1` is ignored, and `c` evaluates to `8`. Understanding why this is can be a bit complicated. The steps are as follows:

* The `!1` in in the first link of `b` becomes the `3` that it sees from the previous link in `c`.
* The `!1` in `b`'s call to `a` becomes the `3` that it sees from the previous link in `b`.
* The `!2` in `b`'s call to `a` becomes the `3` that it sees from the previous link in `c`.
* The `!3` in `b`'s call to `a` becomes the `2` that it sees from two links back in `c`.
* `b`'s call to `a` calculates `3 + 3 + 2 = 8`.
* The `!1` in in the third link of `b` becomes the `8` that it sees from the previous link. This is the last link in `b`, so `b` evaluates to `8`.
* `c`'s call to `b` is in `c`'s last link, so `c` also evaluates to `8`.

```
a: !1 + !2 + !3
b: !1 -> !1 -> a
c: 1 -> 2 -> 3 -> b
```

In the program above, `a` is called from the third link in `b`, so `b` now looks at the values passed to it from only the previous link in 'c'. `1` and `2` are ignored, and `c` evaluates to `9`. You can try to figure our why on your own. If you are having trouble understanding how chain calls and backlinks work, the next section introduces a concept which may make it easier.
