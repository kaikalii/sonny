# Notes

Generating a tone is all well and good, but if you want to actually make a song, you will probably want to write notes. Notes are notated with both a **frequency** and a **duration**.

Lets modify our example from the previous tutorial.

```
std gen

output: A4 -> gen::sine -> out: 5
```

This program will produce a tone with a duration that is as long as the number you specify after the `out` term, in this case, 5 seconds. However, this number can only specify the duration of a single note for the entire project. If we want more notes, then we use a special note list syntax.

```
std gen

output: {A4:5} -> gen::sine -> out
```

This will produce identical output to the first program. However, with the note list syntax, we can specify more notes after the first one.

```
std gen

output: {A4:2, C#:2} -> gen::sine -> out
```

This will play an A4 note for 2 seconds followed by a C#4 for 2 seconds. You can put as many notes in the list as you like.

```
std gen

output: {A4:2, C#:2, E:1, C#:1, A:2} -> gen::sine -> out
```

The first note in the list should have its octave specified. All subsequent notes will be in the same octave until you specify a different one. This means that note lists like `{A4:1, C#:1, G#3:1, E:1}` and `{A4:1, C#4:1, G#3:1, E3:1}` are identical.

As before, you can specify a note's frequency as a number of hertz rather than a musical note name.

```
{100:2, 235.75:1, 311.32:1}
```

The above examples specify the note's duration as a number of seconds, but it can also be specified as a musical note duration or as a fraction of four beats. When defining note durations in this way, Sonny takes the project's tempo into account. The default tempo is 120 beats per minute, but you can set it using the `tempo` keyword.

```
tempo: 60

{A4:1, 440:q, A4:1/4}
```

The notes in the list above are all identical. The `q` indicates a quarter note duration. You can use `w`, `h`, `q`, `e`, `s`, and `ts` to refer to whole, half, quarter, eighth, sixteenth, and thirty-second beat durations respectively. You can also dot these specifiers to indicate dotted notes. This means that durations like `e.` and `3/16` or `h..` and `7/8` are identical.

Let's tie off this section by combining everything.

```
std gen

tempo: 112

output: { Eb4:q., Bb3:e, Eb4:q., Bb3:e, Eb4:e, Bb3:e, Eb4:e, G:e, Bb:h } -> gen::sin -> out
```

This plays the first two measures of Mozart's *Eine kleine Nachtmusik* at 112 bpm.

Sonny interprets newlines the same as spaces, so if it makes it easier to read, you can split the last line into multiple.

```
std gen

tempo: 112

output:
    { Eb4:q., Bb3:e, Eb4:q., Bb3:e, Eb4:e, Bb3:e, Eb4:e, G:e, Bb:h } ->
    gen::sin -> out
```
