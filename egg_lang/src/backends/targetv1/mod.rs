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
    pub main_file_rules: Stack<StringOps>,
    pub struct_rules: StructRules,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StringOps {
    ModuleName,
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

                let mut file_name = String::new();

                for op in self.main_file_rules.ops.iter() {
                    match op {
                        StringOps::Concat { value } => file_name.push_str(value),
                        StringOps::ModuleName => file_name.push_str(&exe.main_module.file_name),
                    }
                }

                let structs = self.struct_rules.compile(&exe.main_module);

                let contents = format!(
                    r#"
{structs}
"#
                )
                .trim()
                .to_string();

                files.push(File {
                    contents,
                    file_name,
                })
            }
            ArtifactType::Library(_) => todo!(),
        }

        files
    }
}
