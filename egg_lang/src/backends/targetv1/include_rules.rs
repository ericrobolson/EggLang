use crate::intermediate_representation::Module;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IncludeRules {
    pub ops: Vec<IncludeOps>,
}

/// The operations supported for building the struct body.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum IncludeOps {}

impl IncludeRules {
    /// Compiles the given module's structs.
    pub fn compile(&self, module: &Module) -> String {
        String::new()
    }
}
