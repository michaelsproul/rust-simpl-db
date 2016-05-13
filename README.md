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
