
# Testing

There are three types of tests included: doc tests, unit tests & integration tests

## To only run the Rust Docs tests
```zsh
cargo test --doc
```

## To only run the unit tests, filter by test names `unit_` This is a bit of a workaround, but it works
```zsh
cargo test unit_
```

## To run all the tests
Note: This requires a test [DB Setup](db.md#integration-testing). It must be setup first. 
```zsh
cargo test
```