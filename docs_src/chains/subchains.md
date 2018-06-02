# Subchains

Let's say we want to add together two sine waves of different frequencies. We can make a simple adder like this:

```
5 -> 7 -> (!1 + !2)
```
The above chain evaluates to `12`.

We can then replace the `5` and `7` with sine wave generators of different frequencies and make the chain output.

```
std gen

output: A4 -> gen::sine -> C#5 -> gen::sine -> (!1 + !2) -> out: 5
```
Simply replacing the numbers may seem like it will work, and this code will compile. However, it will not generate the expected output because we have introduced some ambiguity. The `(!1 + !2)` link is not adding the outputs of the sine generator links. `!1` becomes the generated `C#5` wave like it should, but `!2` becomes the frequency value of `C#5` which is roughly `277.183`. Adding this number to every sample from the generated `C#5` wave will cause every sample to be somewhere in the range of `277.183 +- 1`. When finally evaluated, sample values greater than `1` become `1` and those less than `-1` become `-1`, so the program above will generate an audio buffer of all `1`'s, which has no oscillation and is thus silent.

To remove this ambiguity, we enclose the wave-generating links in a **subchain** by using subchain delimeters `| |`.

```
std gen

output: |A4 -> gen::sine| -> |C#5 -> gen::sine| -> (!1 + !2) -> out: 5
```

To make this more readable, you can put each section on a new line if you like.

```
std gen

output:
    |A4 -> gen::sine| ->
    |C#5 -> gen::sine| ->
    (!1 + !2) ->
    out: 5
```

We have now specified that `A4 -> gen::sine` and `C#5 -> gen::sine` are in their own chains, so they are treated by everything outside the subchain delimeters as a chain call rather than a sequence of links. `!1` becomes the generated `C#5` wave, like it did before, but `!2` now becomes the generated `A4` wave.

We are almost done adding the waves. If we compile the above program as is, the two waves will constructively interfere in certain places, creating samples with an amplitude greater than `1` or less than `-1`. Because we are adding waves, we have to normalize the sum by dividing by the number of waves we are adding.

```
std gen

output:
    |A4 -> gen::sine| ->
    |C#5 -> gen::sine| ->
    (!1 + !2) / 2 ->
    out: 5
```

Compiling this should be generate a nice A major chord.

One last note about subchains. They can be named and then called later.

```
math: |add: !2 + !1| -> |subtract: !2 - !1|

1 -> 2 -> math::add         # 1 + 2 = 3
7 -> 3 -> math::subtract    # 7 - 3 = 4
```

Calling the `math` chain above itself would be kind of nonsensical, but it can be done.

```
3 -> 2 -> math      # 2 - (3 + 2) = -3
```
