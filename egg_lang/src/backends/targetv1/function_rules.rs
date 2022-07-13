use crate::intermediate_representation::{Module, Property};

/// Rules for build up a struct.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FuncRules {
    /// Ops used for function declarations. Not used for the actual implementation.
    /// Usage is for C headers and the like.
    pub declaration_ops: Vec<FuncOps>,
    /// The ops used to construct the function and it's body.
    pub implementation_ops: Vec<FuncOps>,
    /// The ops used to construct the properties of the struct.
    pub property_ops: Vec<PropertyOps>,
}

/// The operations supported for building the struct body.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum FuncOps {
    BuildProperties,
    Concat { value: String },
    FuncName,
    ReturnType,
}

/// The operations supported for properties.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyOps {
    Concat { value: String },
    ConcatIfNotEnd { value: String },
    PropertyName,
    PropertyType,
}

impl FuncRules {
    /// Compiles the given module's function definitions.
    pub fn compile_definitions(&self, module: &Module) -> String {
        module
            .funcs
            .iter()
            .map(|f| {
                let mut compiled = String::default();

                for op in self.declaration_ops.iter() {
                    match op {
                        FuncOps::Concat { value } => compiled.push_str(value),
                        FuncOps::FuncName => compiled.push_str(&f.name),
                        FuncOps::BuildProperties => {
                            compiled.push_str(&self.compile_properties(&f.properties))
                        }
                        FuncOps::ReturnType => compiled.push_str(&f.return_type.to_string()),
                    }
                }
                compiled
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Compiles the given module's function implementations.
    pub fn compile_implementations(&self, module: &Module) -> String {
        module
            .funcs
            .iter()
            .map(|f| {
                let mut compiled = String::default();

                for op in self.implementation_ops.iter() {
                    match op {
                        FuncOps::Concat { value } => compiled.push_str(value),
                        FuncOps::FuncName => compiled.push_str(&f.name),
                        FuncOps::BuildProperties => {
                            compiled.push_str(&self.compile_properties(&f.properties))
                        }
                        FuncOps::ReturnType => compiled.push_str(&f.return_type.to_string()),
                    }
                }
                compiled
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Compile the properties for the struct.
    fn compile_properties(&self, properties: &Vec<Property>) -> String {
        let end_idx = if properties.is_empty() {
            0
        } else {
            properties.len() - 1
        };

        properties
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let mut property = String::default();
                for op in self.property_ops.iter() {
                    match op {
                        PropertyOps::Concat { value } => property.push_str(value),
                        PropertyOps::PropertyType => property.push_str(&p.t.to_string()),
                        PropertyOps::PropertyName => property.push_str(&p.name),
                        PropertyOps::ConcatIfNotEnd { value } => {
                            if idx != end_idx {
                                property.push_str(value)
                            }
                        }
                    }
                }
                property
            })
            .collect::<Vec<String>>()
            .join("")
    }
}
