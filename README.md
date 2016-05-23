## Interesting Bits

* The logic for growing the table is in `relation.rs`, function `grow`.
* The logic for generating page IDs from a partially specified hash is in `partial_hash.rs`,
  as the `PageIdIter` iterator.

## Building

Run `make` (assuming you still have a Rust compiler of some kind installed).

You can then use the shell scripts `create`, `insert`, `select`, etc.

## Testing

You can run our tests using `cargo test`. Some of the QuickCheck ones might take a while!

## Logging

Enable via the `RUST_LOG` env variable, see:

http://rust-lang-nursery.github.io/log/env_logger/

For example:

```
# Bash.
$ RUST_LOG=trace ./create

# Fish.
$ env RUST_LOG=trace ./create
```

To enable logging for a new front-end binary use `run_main`.
