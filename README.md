# Rejects

> This project is still rough around the edges and without much documentation. However, the examples in this README are fully functional. The main reason behind this crate was to act as the backbone to another crate I'm writing called [reflex](https://github.com/RRethy/reflex) which is a lexical analyzer generator implemented as a Rust procedural macro.

## Usage

```rust
use rejects::make_rejects;
let re = make_rejects!(r"[a-z]+[0-9]?(\d\d\d\d|abcd)*");
println!(
    "{}",
    re.find_end("abcdefg11234abcd1234this is no longer getting matched")
);
println!("{}", re.find_end("abcdefhijk234"));
```

Will print (indices are inclusive)

```
19
-1
```

Currently, a `Rejects` object only exposes `find_end` to return the inclusive index of the last matched utf-8 character.

`rejects::rejects::Rejects::new(r"pat")` is functionally equivalent to `rejects::make_rejects!(r"pat");`, except the latter will compile the automaton at compile time while the former will do so at run time.

You will need to add the following to your Cargo.toml since this is not yet a published crate.

```toml
[dependencies]
rejects = { git = "https://github.com/rrethy/rejects" }
```
