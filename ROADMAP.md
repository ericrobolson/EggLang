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

## C Backend 
- [x] As a developer, I want to be able to specify operations for the C backend to construct structs.
- [x] As a developer, I want my struct definitions to be in headers
- [x] As a developer, I want my function definitions for C to be in headers
- [x] As a developer, I want my function implementations to be in implementation files
- [x] As a developer, I want my structs and function definitions to be imported in my implementation files
- [ ] As a developer, I want the ability to add in expressions to function implementations
- [ ] As a developer, I want the ability to specify comments
- [ ] As a developer, I want the ability to use structs in functions
- [ ] As a developer, I want the ability to specify built in primitive types and functions
- [ ] As a developer, I want the ability to include the primitive built in type headers

## JavaScript backend
- [ ] Todo

## C# backend
- [ ] Todo

## Compiler Front end
- [ ] 

## Design
- [ ] As a developer, I want the ability to define structs
- [ ] As a developer, I want the ability to define enums
- [ ] As a developer, I want the ability to define function
- [ ] As a developer, I want generics

## Extension + useability
- [ ] As a developer, I want to be able to use a language server that can run tests, do code coverage integrations, code completion, and run tests. It should come with built in benchmarking, etc. It should also be bootstrapped in `EggLang`. 
- [ ] As a developer, I want to expose a C API for the runtime to allow embedding.
- [ ] As a developer, I want to extend `benchy` to support the number of calls made.

