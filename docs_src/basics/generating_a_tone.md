The most basic interesting thing you can do in Sonny is generate a tone.

To generate a tone, we have to specify two of its attributes: its **frequency** and its **waveform**.

To get started, open up a blank document and name it `tutorial.son`.

Sonny gives you the tools to make your own wave generators, but for now, we will use one of the ones included with the standard library. At the top of your file, add:

```
std gen
```

The `std` keyword tells Sonny to use a standard library file. `gen` is the name of the standard wave generator library.

Now for the frequency. For this example, we will generate an A4 note, which is 440 Hz. In Sonny, you can denote frequencies by either the number of hertz or the note name. A couple lines down, add either `A4` or `440`.

```
std gen

A4
```

We have the tone's frequency, now we need to give it a waveform. Let's use the standard library's sine wave generator.

```
std gen

A4 -> gen::sine
```

The `->` operator denotes that we are passing the A4 to the sine generator so that it knows what frequency to use.

We are almost done. All that is left to do is tell Sonny that we want it to play the tone. We do that by add another `->` and the word `out`.

```
std gen

A4 -> gen::sine -> out
```

That's it! Save your file, and from the terminal, navigate to its folder. Run the command:
```
sonny tutorial.son --play
```
After a brief moment, you computer's default audio player should pop up and play the tone for one second. Make sure your sound is on, or you might miss it.

You may have noticed that the sound file is called `anonXXXX.wav`. This is because we did not name our line with the tone. Go back to your `tutorial.son` file and add a name.

```
std gen

output: A4 -> gen::sine -> out
```

The tone only played for one second because that is the default time that Sonny uses if it does not know how long an audio file should be. To increase this time, add a number after the `out`.

```
std gen

output: A4 -> gen::sine -> out: 5
```

If you run `sonny tutorial.son --play` again, this time the tone should last for 5 seconds, and the generated file should be named `output.wav`.

That's all there is to it. Next, we'll discuss how to program in [notes](https://github.com/kaikalii/sonny/wiki/Notes) so that we can make sounds that are a little more interesting.
