#[cfg(test)]
mod tests {
    use crate::{
        backends::file::File,
        intermediate_representation::{self, *},
    };

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
                },
            }),
        };

        let main_c = r#"
struct FooBar {
} FooBar;

struct TestyMctest {
    EggBool alive;
    EggI32 hp; 
} TestyMctest;
"#;

        let expected = vec![File {
            contents: main_c.trim().to_string(),
            file_name: "main.c".into(),
        }];

        let actual = target().compile(input);

        assert_eq!(expected, actual)
    }
}
