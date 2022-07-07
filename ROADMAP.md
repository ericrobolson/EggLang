# The roadmap for this project.
## Tokenizer
- [x] As a user, I want to be able to tokenize strings.
- [x] As a user, I want to be able to tokenize identifiers.
- [x] As a user, I want to be able to tokenize f64s. I also want to make sure identifiers don't start with numbers.
- [x] As a user, I want to be able to tokenize comments.
- [x] As a user, I want to add in symbols and end things on `[]`, `()` or `{}`
- [x] As a developer I want to minimize the exposed surface area of the tokenizer and break up the module into logical parts.
## Direction
- [x] As a core developer, I need to decide on a Lisp or a Forth or a map based pattern matching language like Elixir/Erlang. Map based concatenative language like Forth/Elixir.
- [ ] As a core developer, I need to decide whether to use types or not
## Extension
- [ ] As a developer, I want to be able to use a language server that can run tests, do code coverage integrations and run tests. It should come with built in benchmarking, etc. It should also be bootstrapped in `EggLang`. 

## Benchmarks
- [ ] As a developer, I want to extend `benchy` to support the number of calls made.

## Future
- [ ] As a core developer, I need to expose a C API for the runtime to allow embedding.
