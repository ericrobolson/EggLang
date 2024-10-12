use crate::definition::type_::Type;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub location: lisper::Location,
    pub name: String,
    pub type_: Type,
}
impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.type_ == other.type_
    }
}
