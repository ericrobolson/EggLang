# The roadmap for this project.
## Tokenizer
- [x] As a user, I want to be able to tokenize strings.
- [x] As a user, I want to be able to tokenize identifiers.
- [x] As a user, I want to be able to tokenize f64s. I also want to make sure identifiers don't start with numbers.
- [x] As a user, I want to be able to tokenize comments.
- [x] As a user, I want to add in symbols and end things on `[]`, `()` or `{}`
- [x] As a developer I want to minimize the exposed surface area of the tokenizer and break up the module into logical parts.

## Parser
- [x] As a user, I need to be able to parse lists.
- [x] As a user, I need to be able to parse things other than lists.

## Design
- [ ] As a developer, I want the ability to define structs
- [ ] As a developer, I want the ability to define enums
- [ ] As a developer, I want the ability to define function
- [ ] As a developer, I want generics

## Extension + useability
- [ ] As a developer, I want to be able to use a language server that can run tests, do code coverage integrations, code completion, and run tests. It should come with built in benchmarking, etc. It should also be bootstrapped in `EggLang`. 
- [ ] As a developer, I want to expose a C API for the runtime to allow embedding.
- [ ] As a developer, I want to extend `benchy` to support the number of calls made.

