# brainfsym

Rust library for symbolically executing brainf\*\*\* code 

## Disclaimer

The basic library functionality works and the tests pass.
However, this project is abandoned for reasons stated below.

### Brainf\*\*\* doesn't symbolically execute well

Brainf\*\*\* as a language is apparently not suited to symbolic execution,
because most computation is done with branches.
This means individual constraints passed to [z3] are really simple.
So symbolic execution of brainf\*\*\* turns out to just be overcomplicated fuzzing.

### Difficulty linking [z3] with [Yew]

This project uses the [z3] SMT solver.
The intention was to have a nice web UI with [Yew]
for visualizing the paths and constraints,
compiled to [WASM] as a static web app.
However, linking C++ code (z3) with Rust and compiling the result to [WASM]
is apparently really hard and isn't really a solved problem.

[z3]: https://github.com/prove-rs/z3.rs
[Yew]: https://yew.rs
[WASM]: https://webassembly.org/
