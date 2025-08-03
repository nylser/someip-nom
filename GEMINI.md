# GEMINI.md

## Project Overview

This project is a parser for the SOME/IP protocol, written in Rust. It utilizes the `nom` parser combinator library to define and parse the SOME/IP message format. The main logic is contained in `src/main.rs`, which defines the data structures for SOME/IP messages and the parsing functions.

The project is in its early stages, with the basic structure for parsing SOME/IP headers and some data types in place.

## Building and Running

As a standard Rust project using Cargo, the following commands can be used:

*   **Build:**
    ```bash
    cargo build
    ```

*   **Run:**
    ```bash
    cargo run
    ```

*   **Test:**
    ```bash
    cargo test
    ```
    *(Note: No tests have been implemented yet.)*

## Development Conventions

*   **Formatting:** The project should adhere to standard Rust formatting, which can be enforced using `rustfmt`:
    ```bash
    cargo fmt
    ```
*   **Linting:** The `clippy` tool can be used to catch common mistakes and improve the code:
    ```bash
    cargo clippy
    ```
