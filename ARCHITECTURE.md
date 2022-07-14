# Decisions
- Make both a CLI and a library/c library for this
- Use Rust as a core
- Easily embeddable on all platforms
- Bottom up development. Take a user story, then focus on building up what you need from the bottom (lowest component) up. E.g. need to get a webpage's contents? Start with the HTTP client, then the response parser, then the application logic after the others are done.
- I have decided to go with a concatenative language. I'm going with a pattern matching language with a stack.

# Data Types
- f64s for numbers


# Project Structure
- `src/` - Contains all code
- - `backend/` - Contains all code related to compiler backends. This module does no error checking and instead takes in an intermediate representation and compiles that directly.
- - `frontend/` - Contains all code related to the compiler frontend. This module handles tokenization, parsing, lexing and syntactic analysis. In addition it converts the source input into an intermediate representation the backend can compile.
- - `intermediate_representation` - Contains the bare minimum data structures required to target a backend. Heavily inspired by C with functions and structs as the primary primitives.
- `backends/` - Contains JSON implementations for backends.
