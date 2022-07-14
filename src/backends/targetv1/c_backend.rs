#[cfg(test)]
mod tests {
    use crate::{backends::file::File, intermediate_representation::*};

    use super::super::*;

    fn target() -> TargetV1 {
        let json = std::fs::read_to_string("backends/backend_c.json").unwrap();

        TargetV1::deserialize(&json).unwrap()
    }

    #[test]
    fn simple_hello_world() {
        let input = Artifact {
            artifact_type: ArtifactType::Executable(Executable {
                file_name: "hello_world".to_string(),
                main_module: Module {
                    file_name: "main".into(),
                    funcs: vec![Func {
                        name: "say_hello".into(),
                        properties: vec![],
                        return_type: PrimitiveType::EggBool,
                    }],
                    structs: vec![
                        Struct {
                            name: "FooBar".into(),
                            properties: vec![],
                        },
                        Struct {
                            name: "TestyMctest".into(),
                            properties: vec![
                                Property {
                                    name: "alive".into(),
                                    t: PrimitiveType::EggBool,
                                },
                                Property {
                                    name: "hp".into(),
                                    t: PrimitiveType::EggI32,
                                },
                            ],
                        },
                    ],
                    includes: vec![],
                },
            }),
        };

        let main_h = "
EggBool say_hello();

struct FooBar {
} FooBar;

struct TestyMctest {
\tEggBool alive;
\tEggI32 hp;
} TestyMctest;";

        let main_c = "
#include \"main.h\"

EggBool say_hello() {
}
";

        let expected = vec![
            File {
                contents: main_c.trim_start().to_string(),
                file_name: "main.c".into(),
            },
            File {
                contents: main_h.trim_start().to_string(),
                file_name: "main.h".into(),
            },
        ];

        let actual = target().compile(input);

        assert_eq!(expected, actual)
    }

    #[test]
    fn func_generation_no_args() {
        let input = Artifact {
            artifact_type: ArtifactType::Executable(Executable {
                file_name: "hello_world".to_string(),
                main_module: Module {
                    file_name: "main".into(),
                    funcs: vec![Func {
                        name: "say_hello".into(),
                        properties: vec![],
                        return_type: PrimitiveType::EggBool,
                    }],
                    structs: vec![],
                    includes: vec![],
                },
            }),
        };

        let main_h = "
EggBool say_hello();\n\n";

        let main_c = "
#include \"main.h\"

EggBool say_hello() {
}\n";

        let expected = vec![
            File {
                contents: main_c.trim_start().to_string(),
                file_name: "main.c".into(),
            },
            File {
                contents: main_h.trim_start().to_string(),
                file_name: "main.h".into(),
            },
        ];

        let actual = target().compile(input);

        assert_eq!(expected, actual)
    }

    #[test]
    fn func_generation_with_args() {
        let input = Artifact {
            artifact_type: ArtifactType::Executable(Executable {
                file_name: "hello_world".to_string(),
                main_module: Module {
                    file_name: "main".into(),
                    funcs: vec![Func {
                        name: "say_hello".into(),
                        properties: vec![
                            Property {
                                name: "bar".into(),
                                t: PrimitiveType::EggBool,
                            },
                            Property {
                                name: "foo".into(),
                                t: PrimitiveType::EggI32,
                            },
                        ],
                        return_type: PrimitiveType::EggBool,
                    }],
                    structs: vec![],
                    includes: vec![],
                },
            }),
        };

        let main_h = "
EggBool say_hello(EggBool bar, EggI32 foo);\n\n";

        let main_c = "
#include \"main.h\"

EggBool say_hello(EggBool bar, EggI32 foo) {
}\n";

        let expected = vec![
            File {
                contents: main_c.trim_start().to_string(),
                file_name: "main.c".into(),
            },
            File {
                contents: main_h.trim_start().to_string(),
                file_name: "main.h".into(),
            },
        ];

        let actual = target().compile(input);

        assert_eq!(expected, actual)
    }

    #[test]
    fn func_generation_multiple() {
        let input = Artifact {
            artifact_type: ArtifactType::Executable(Executable {
                file_name: "hello_world".to_string(),
                main_module: Module {
                    file_name: "main".into(),
                    funcs: vec![
                        Func {
                            name: "first_fn".into(),
                            properties: vec![Property {
                                name: "derp".into(),
                                t: PrimitiveType::EggBool,
                            }],
                            return_type: PrimitiveType::EggI32,
                        },
                        Func {
                            name: "say_hello".into(),
                            properties: vec![
                                Property {
                                    name: "bar".into(),
                                    t: PrimitiveType::EggBool,
                                },
                                Property {
                                    name: "foo".into(),
                                    t: PrimitiveType::EggI32,
                                },
                            ],
                            return_type: PrimitiveType::EggBool,
                        },
                    ],
                    structs: vec![],
                    includes: vec![],
                },
            }),
        };

        let main_h = "
EggI32 first_fn(EggBool derp);

EggBool say_hello(EggBool bar, EggI32 foo);\n\n";

        let main_c = "
#include \"main.h\"

EggI32 first_fn(EggBool derp) {
}

EggBool say_hello(EggBool bar, EggI32 foo) {
}\n";

        let expected = vec![
            File {
                contents: main_c.trim_start().to_string(),
                file_name: "main.c".into(),
            },
            File {
                contents: main_h.trim_start().to_string(),
                file_name: "main.h".into(),
            },
        ];

        let actual = target().compile(input);

        assert_eq!(expected, actual)
    }
}
