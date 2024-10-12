use std::collections::HashMap;

use lisper::{Error, List};

use super::type_::Type;

/// A field. Can be properties, arguments, etc.
#[derive(Debug, Clone)]
pub struct Field {
    pub location: lisper::Location,
    pub name: String,
    pub type_: Type,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.type_ == other.type_
    }
}
impl Field {
    pub fn parse_field(
        field_type: &str,
        list: &mut List,
        fields: &mut HashMap<String, Self>,
    ) -> Result<(), Error> {
        let mut property_list = list.pop_list(&format!("{} list", field_type))?;
        let (ty_, loc) = property_list.pop_identifier("type")?;
        let ty = Type::try_parse(&ty_, loc)?;

        let (name, loc) = property_list.pop_identifier("name")?;
        match fields.get(&name) {
            Some(_) => {
                return Err(Error {
                    message: format!("Duplicate {} '{}'", field_type, name),
                    location: loc,
                });
            }
            None => {
                fields.insert(
                    name.clone(),
                    Self {
                        location: loc,
                        name,
                        type_: ty,
                    },
                );
            }
        }

        Ok(())
    }
}
