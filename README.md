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

# Import your vocab from duolingo json and get translations
[How to Import your Vocab from DuoLingo](docs/imports.md)

# Testing
[Running Tests](docs/testing.md)

# For Developers
[Developer Setup](docs/developers.md)

# Running
> cargo run

_Note: This is only a command line interface for now_

# Releasing
> cargo build --release

After the build completes, you can find the compiled binary executables within the `target/release` directory inside your project folder.


#Future Considerations
1. Create a GUI to display Memory Cards
2. Add an embedded database as a default option. 
