use super::{PrimitiveType, Property};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Func {
    pub name: String,
    pub properties: Vec<Property>,
    pub return_type: PrimitiveType,
}
