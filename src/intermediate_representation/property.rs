use super::PrimitiveType;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub name: String,
    pub t: PrimitiveType,
}
