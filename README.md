## EggLang
A lispy language that compiles to CPP.

Right now it's a WIP and not setup properly. This repo is more of a snapshot for ideas.

## Compiler Roadmap

Compiler Roadmap

- [x] Add in parsing of outputs for compiler
- [x] Add in int types
- [x] Add in uint types
- [x] Add in compiling of code for compiler
- [x] Generate C++
- [x] Add equality and inequality operators == and !=. Support const types and add in & for referencing the other object
- [x] Add in constructors and destructors to allocate and deallocate memory
- [x] Fix dependency ordering issue. E.g. TypeId
- [x] Fix in equality issue with pointers. Need to do `*a == *b` instead of `a == b` I think
- [x] Add in a 'clone' method to clone objects
- [x] Add in a 'copy_to' method to copy objects
- [x] Implement copying of values
- [x] Implement copying for strings
- [x] Implement copying for vectors
- [x] Implement copying of pointer based objects
- [x] Add in copy assignment operator
- [x] Add in copy constructor - need to add call to normal constructor first.
- [x] Output both a hpp and cpp file, split things up.
- [x] Split up hpp and cpp code
- [x] Redo generating class methods. Note: inline must be on implementation if doing it.
- [x] Migrate each class and function to a separate file
- [x] Fix cpp makefile
- [x] Add custom struct functions to data model
- [x] Add generation of custom struct functions to compiler (header and cpp)
- [x] For compilation, if custom definition doesn't exist, generate it. If it does exist, splice it in. Put all autogenerated stuff at the bottom of the file.
- [ ] Add consts to functions for both self and params
- [ ] Add in print operators for classes
- [ ] Add serialization + deserialization from string? Into a lisp like language?
- [ ] Add ability to add custom functions to structs, such as `collides` for aabbs. Make sure that if the definition is modified, it doesn't delete the custom functions. Alternatively output a `definition.gen` file that can be copy/pasta'd by the user. Or even output a list of comments for it.
- [ ] Add ADT support, can then transform them to structs with methods for matching?
- [ ] Add in ability to use Godot and import the CPP code [See this article](https://docs.godotengine.org/en/stable/contributing/development/core_and_modules/custom_modules_in_cpp.html#doc-custom-modules-in-cpp)
- [ ] Determine if I want to do C style C++ or actual C++ and modify compiler
- [ ] Add in sized arrays?
- [ ] Generate JS?
- [ ] Generate Godot GDScript?
