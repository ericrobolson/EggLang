use crate::intermediate_representation::Module;

/// Rules for building includes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IncludeRules {
    pub ops: Vec<IncludeOps>,
}

/// The operations supported for building the includes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum IncludeOps {
    Concat { value: String },
    IncludeName,
}

impl IncludeRules {
    /// Compiles the given module's structs.
    pub fn compile(&self, module: &Module) -> String {
        module
            .includes
            .iter()
            .map(|i| {
                //
                let mut include = String::new();
                for op in self.ops.iter() {
                    match op {
                        IncludeOps::Concat { value } => include.push_str(value),
                        IncludeOps::IncludeName => include.push_str(i),
                    }
                }
                include
            })
            .collect::<Vec<String>>()
            .join("")
    }
}
