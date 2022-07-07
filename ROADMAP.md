# The roadmap for this project.
## Tokenizer
- [x] As a user, I want to be able to tokenize strings.
- [x] As a user, I want to be able to tokenize identifiers.
- [x] As a user, I want to be able to tokenize f64s. I also want to make sure identifiers don't start with numbers.
- [ ] As a user, I want to be able to tokenize comments.
- [ ] As a user, I want to add in symbols and end things on `[]`, `()` or `{}` or comments
- [ ] As a developer I want to minimize the exposed surface area of the tokenizer and break up the module into logical parts.
## Direction
- [ ] As a core developer, I need to decide on a Lisp or a Forth or a map based language like Elixir/Erlang
- [ ] As a core developer, I need to decide whether to use types or not

## Benchmarks
- [ ] As a developer, I want to extend `benchy` to support the number of calls made.

## Future
- [ ] As a core developer, I need to expose a C API for the runtime to allow embedding.
