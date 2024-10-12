use lisper::{Error, Location};
use type_::Type;

pub mod enum_;
pub mod field;
pub mod function;
pub mod output;
pub mod struct_;
pub mod type_;

pub trait FromLisp: Sized {
    /// The identifier of the type.
    fn identifier() -> &'static str;

    /// Returns a list of related types.
    fn get_related_types(&self) -> Vec<(Location, Type)>;

    /// Converts a lisp list into the type.
    fn from_lisp(mut list: lisper::List) -> Result<Self, Error> {
        let location = list.location();

        // Pop identifier and assert it matches
        match list.pop_identifier("Expected type") {
            Ok((id, location)) => {
                if id != Self::identifier() {
                    return Err(Error {
                        message: format!("Expected identifier '{}'", Self::identifier()),
                        location,
                    });
                }
            }
            Err(e) => return Err(e),
        }
        Self::parse_values(&mut list)
    }

    /// Parses the values of the list into the type.
    fn parse_values(list: &mut lisper::List) -> Result<Self, Error>;

    /// Returns whether the node should be tried.
    fn can_try(list: &lisper::List) -> bool {
        if let Some(i) = list.peek_front() {
            if let Ok(type_id) = i.as_identifier() {
                return type_id == Self::identifier();
            }
        }
        false
    }
}
