
# Getting setup for development


### Install the diesel_cli
```zsh
cargo install diesel_cli --no-default-features --features postgres
```

### To create the db schema
```zsh
diesel migration run
diesel migration redo # to test the drops, optional
```

### To add another migration change via diesel:
```zsh
diesel migration generate <migration_name>
source .env
diesel migration run
diesel print-schema > src/schema.rs
source test.env  # to run tests
```

To update the diesel schema to match the database
```zsh
diesel print-schema > src/studies
```
