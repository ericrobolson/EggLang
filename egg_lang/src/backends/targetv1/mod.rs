mod c_backend;
mod struct_rules;

use self::struct_rules::StructRules;

use super::file::File;
use crate::intermediate_representation::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stack<Op> {
    pub ops: Vec<Op>,
}
impl<Op> Stack<Op> {
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }
}

/// An series of rules used for constructing a language.
/// Typically operates on stacks for simplicity.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TargetV1 {
    pub system_includes: Vec<String>,
    pub struct_rules: StructRules,
    pub file_generation: Vec<GenerationOps>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GenerationOps {
    File {
        comment: String,
        name_ops: Vec<StringOps>,
        content_ops: Vec<ContentOps>,
    },
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ContentOps {
    StructDefinitions,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StringOps {
    ModuleName,
    FileName,
    Concat { value: String },
}

impl TargetV1 {
    fn deserialize<'a>(target_json: &'a str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(target_json)
    }

    pub fn compile(&self, ir: Artifact) -> Vec<File> {
        let mut files = vec![];
        match ir.artifact_type {
            ArtifactType::Executable(exe) => {
                //

                let structs = self.struct_rules.compile(&exe.main_module);

                for op in self.file_generation.iter() {
                    match op {
                        GenerationOps::File {
                            comment: _,
                            name_ops,
                            content_ops,
                        } => {
                            let mut file_name = String::new();

                            for op in name_ops {
                                match op {
                                    StringOps::ModuleName => todo!(),
                                    StringOps::FileName => {
                                        file_name.push_str(&exe.main_module.file_name)
                                    }
                                    StringOps::Concat { value } => file_name.push_str(value),
                                }
                            }

                            let mut contents = String::new();
                            for op in content_ops {
                                match op {
                                    ContentOps::StructDefinitions => contents.push_str(&structs),
                                }
                            }

                            files.push(File {
                                contents,
                                file_name,
                            })
                        }
                    }
                }
            }
            ArtifactType::Library(_) => todo!(),
        }

        files.sort_by(|a, b| a.file_name.partial_cmp(&b.file_name).unwrap());

        files
    }
}
