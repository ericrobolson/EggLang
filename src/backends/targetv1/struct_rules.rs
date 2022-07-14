use crate::intermediate_representation::{Module, Property};

/// Rules for build up a struct.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructRules {
    /// The ops used to construct the struct and it's body.
    pub ops: Vec<StructOps>,
    /// The ops used to construct the properties of the struct.
    pub property_ops: Vec<PropertyOps>,
}

/// The operations supported for building the struct body.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StructOps {
    Concat { value: String },
    StructName,
    BuildProperties,
}

/// The operations supported for properties.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyOps {
    Concat { value: String },
    PropertyName,
    PropertyType,
}

impl StructRules {
    /// Compiles the given module's structs.
    pub fn compile(&self, module: &Module) -> String {
        module
            .structs
            .iter()
            .map(|s| {
                let mut compiled = String::default();

                for op in self.ops.iter() {
                    match op {
                        StructOps::Concat { value } => compiled.push_str(value),
                        StructOps::StructName => compiled.push_str(&s.name),
                        StructOps::BuildProperties => {
                            compiled.push_str(&self.compile_properties(&s.properties))
                        }
                    }
                }
                compiled
            })
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    /// Compile the properties for the struct.
    fn compile_properties(&self, properties: &Vec<Property>) -> String {
        properties
            .iter()
            .map(|p| {
                let mut property = String::default();
                for op in self.property_ops.iter() {
                    match op {
                        PropertyOps::Concat { value } => property.push_str(value),
                        PropertyOps::PropertyType => property.push_str(&p.t.to_string()),
                        PropertyOps::PropertyName => property.push_str(&p.name),
                    }
                }
                property
            })
            .collect::<Vec<String>>()
            .join("")
    }
}
