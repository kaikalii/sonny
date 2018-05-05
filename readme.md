# Description

*Sonny* is a functional programming language designed to create music and other sounds. It features a highly modular, fully programmable sound generation and transformation pipeline so that the user has complete control over what is generated.

*Sonny* is currently in heavy development and any help in this endeavor is greatly appreciated! See the contribution guidelines below if you are interested.

# Features

### Current Features

* Standard programming language expression evaluation including arithmetic, logical, and ternary operators
* Modular arithmetic sound transformation via function-like constructs called "chains"
* Easy-to-type note entry to build song loops
* Loop arrangement via chains
* Simple but effective module system for separating code into multiple files or libraries
* Compiles to .WAV format

### Planned Features

* Output to other audio formats, namely .MP3 and .OGG
* Frequency-domain sound manipulation
* Playback and manipulation of external audio samples
* A more powerful module system for more complex libraries
* Continuous sound playback from the compiler itself
* Proper language documentation (currently there is only a lackluster grammar file)
* Getting user input to allow for interactive programs
* A decent standard library with lots of useful chains

# Design Goals

### Goals

The language should (eventually)...

* be capable of creating any audio file that a typical DAW (Digital Audio Workstation) program like Ableton, Reason, or Logic are capable of making.
* be easy to use for the purposes of sound design and audio generation.
* be fast. A decent machine should be capable of continuous audio generation and playback without any pauses.
* have the simplest, nicest-looking syntax possible without sacrificing features. (This is obviously subjective.)
* be compatible with all major desktop operating systems

### Non-Goals

The does not need to, and probably should not...

* be easy to use as a general-purpose programming language. There are plenty of very good general-purpose programming languages already. *Sonny* only seeks to be good in its niche of sound processing.

# Installation

The *Sonny* compiler is written in the Rust programming language, and releases are uploaded to Rust's package website [crates.io](http://crates.io). Before you can install *Sonny* you will need to install Rust's package manager, cargo.

If you do not already have *Rust* installed, head over to [the *Rust* website](https://www.rust-lang.org/) and install it. Once cargo is installed, simply run this command in your terminal:
```
cargo install sonny
```
Then, you can run the example:
```
sonny example.son --play
```

# Tutorial

To learn the basics or programming is *Sonny*, head over to [the wiki]().

# Contributing

Any contribution to *Sonny*'s development is greatly appreciated. If you would like to contribute, please keep the following in mind:

* Unless your contribution is a simple bug fix, open an issue to discuss it before submitting a pull-request.
* Please use rustfmt on your code with the default settings.
