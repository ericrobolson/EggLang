mod c_backend;
mod function_rules;
mod include_rules;
mod struct_rules;

use self::{function_rules::FuncRules, include_rules::IncludeRules, struct_rules::StructRules};
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
    pub struct_rules: StructRules,
    pub file_generation: Vec<GenerationOps>,
    pub func_rules: FuncRules,
    pub include_rules: IncludeRules,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GenerationOps {
    File {
        comment: String,
        content_ops: Vec<ContentOps>,
        name_ops: Vec<StringOps>,
    },
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ContentOps {
    Concat { value: String },
    FunctionDefinitions,
    FunctionImplementations,
    StructDefinitions,
    IncludeRules,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StringOps {
    Concat { value: String },
    FileName,
    ModuleName,
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
                let module = &exe.main_module;

                let structs = self.struct_rules.compile(module);
                let func_definitions = self.func_rules.compile_definitions(module);
                let func_implementations = self.func_rules.compile_implementations(module);
                let includes = self.include_rules.compile(module);

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
                                    ContentOps::FunctionDefinitions => {
                                        contents.push_str(&func_definitions)
                                    }
                                    ContentOps::FunctionImplementations => {
                                        contents.push_str(&func_implementations)
                                    }
                                    ContentOps::Concat { value } => contents.push_str(&value),
                                    ContentOps::IncludeRules => contents.push_str(&includes),
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
