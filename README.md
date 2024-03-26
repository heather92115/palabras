# Simple, free and open source word and phrase pairs to learn a new language.

## Prerequisites 
1. Rust
2. Cargo
3. Docker(optional)
4. Postgres database

This project is created with Rust and Cargo.
To build the project:
> cargo build

To update the project:
> cargo update

# Database Setup
[DB Setup](docs/db.md)

# Testing
[Running Tests](docs/testing.md)

# For Developers
[Developer Setup](docs/developers.md)

# Running the CLI
A database connection is required, see [DB Setup](docs/db.md)
> cargo run --bin shell_study

# Running the GQL Web Application Server

Make certain you set up your TCP Address to be used.
A database connection is required, see [DB Setup](docs/db.md)
> export SERVER_ADDR=0.0.0.0:3000
> cargo run

The GraphiQL IDE can be found on the root TCP Address

# Exporting the GQL Schema
> cargo run --bin export_gql_schema

# Releasing
> cargo build --release

After the build completes, you can find the compiled binary executables within the `target/release` directory inside your project folder.


#Future Considerations
1. Create a GUI to display Memory Cards
2. Add an embedded database as a default option. 
