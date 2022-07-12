use super::PrimitiveType;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Struct {
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub name: String,
    pub t: PrimitiveType,
}
