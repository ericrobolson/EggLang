use crate::intermediate_representation::Module;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructRules {
    /// The ops used to construct the struct and it's body.
    pub ops: Vec<StructOps>,
    /// The ops used to construct the properties of the struct.
    pub property_ops: Vec<PropertyOps>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StructOps {
    Concat { value: String },
    StructName,
    BuildProperties,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyOps {
    Concat { value: String },
    PropertyType,
    PropertyName,
}

impl StructRules {
    pub fn compile(&self, module: &Module) -> String {
        module
            .structs
            .iter()
            .map(|s| {
                let mut compiled = String::default();
                let properties = {
                    s.properties
                        .iter()
                        .map(|p| {
                            let mut property = String::default();
                            for op in self.property_ops.iter() {
                                match op {
                                    PropertyOps::Concat { value } => property.push_str(value),
                                    PropertyOps::PropertyType => todo!(),
                                    PropertyOps::PropertyName => property.push_str(&p.name),
                                }
                            }
                            property
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                };

                for op in self.ops.iter() {
                    match op {
                        StructOps::Concat { value } => compiled.push_str(value),
                        StructOps::StructName => compiled.push_str(&s.name),
                        StructOps::BuildProperties => compiled.push_str(&properties),
                    }
                }
                compiled
            })
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
