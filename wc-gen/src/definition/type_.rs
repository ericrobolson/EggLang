use super::Error;
use lisper::Location;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
    String,
    Float,
    Void,
    Identifier(String),
    List(Box<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Float => write!(f, "f32"),
            Type::Void => write!(f, "void"),
            Type::Identifier(name) => write!(f, "{}", name),
            Type::List(inner) => write!(f, "{}[]", inner),
        }
    }
}

impl Type {
    pub fn is_identifier(&self) -> bool {
        match self {
            Type::Identifier(_) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            Type::List(_) => true,
            _ => false,
        }
    }

    /// Returns the inner type of the type.
    /// For example, `int[]` would return `int`.
    pub fn inner_type(&self) -> Type {
        match self {
            Type::List(inner) => inner.inner_type(),
            ty => ty.clone(),
        }
    }

    /// Tries to parse a type from a string.
    pub fn try_parse(value: &str, location: Location) -> Result<Self, Error> {
        match value {
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "f32" => Ok(Type::Float),
            "void" => Ok(Type::Void),
            identifier => {
                //
                let first = identifier.chars().nth(0).unwrap();
                if !first.is_alphabetic() && first != '_' {
                    return Err(Error {
                        message: "Identifier must start with a letter or '_'".into(),
                        location: location,
                    });
                }

                if identifier.ends_with("[]") {
                    let inner = &identifier[..identifier.len() - 2];
                    let inner = Type::try_parse(inner, location.clone())?;

                    if inner == Type::Void {
                        return Err(Error {
                            message: "void can not be attached to a list".into(),
                            location: location,
                        });
                    }

                    return Ok(Type::List(Box::new(inner)));
                }
                if identifier.ends_with("[") {
                    return Err(Error {
                        message: "Unclosed list".into(),
                        location: location,
                    });
                }
                Ok(Type::Identifier(identifier.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_type() {
        let ty = Type::I8;
        assert_eq!(ty.inner_type(), Type::I8);

        let ty = Type::List(Box::new(Type::I32));
        assert_eq!(ty.inner_type(), Type::I32);

        let ty = Type::List(Box::new(Type::List(Box::new(Type::I64))));
        assert_eq!(ty.inner_type(), Type::I64);
    }

    #[test]
    fn test_identifier_must_be_alphabetic() {
        let input = "1";
        let expected = "Identifier must start with a letter or '_'".to_string();
        assert_eq!(
            Type::try_parse(input, Location::default()),
            Err(expected.into())
        );

        let input = "[a";
        let expected = "Identifier must start with a letter or '_'".to_string();
        assert_eq!(
            Type::try_parse(input, Location::default()),
            Err(expected.into())
        );

        let input = "_a-struct";
        let expected = Type::Identifier("_a-struct".into());
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));

        let input = "a-struct";
        let expected = Type::Identifier("a-struct".into());
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn test_try_parse() {
        assert_eq!(Type::try_parse("i8", Location::default()), Ok(Type::I8));
        assert_eq!(Type::try_parse("i16", Location::default()), Ok(Type::I16));
        assert_eq!(Type::try_parse("i32", Location::default()), Ok(Type::I32));
        assert_eq!(Type::try_parse("i64", Location::default()), Ok(Type::I64));
        assert_eq!(Type::try_parse("bool", Location::default()), Ok(Type::Bool));
        assert_eq!(
            Type::try_parse("string", Location::default()),
            Ok(Type::String)
        );
        assert_eq!(Type::try_parse("f32", Location::default()), Ok(Type::Float));
        assert_eq!(
            Type::try_parse("a-struct", Location::default()),
            Ok(Type::Identifier("a-struct".into()))
        );
    }

    #[test]
    fn parse_unsigned() {
        let input = "u8";
        let expected = Type::U8;
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));

        let input = "u16";
        let expected = Type::U16;
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));

        let input = "u32";
        let expected = Type::U32;
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));

        let input = "u64";
        let expected = Type::U64;
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn parse_void() {
        let input = "void";
        let expected = Type::Void;
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn parse_void_list_returns_err() {
        let input = "void[]";
        let expected = "void can not be attached to a list".to_string();
        assert_eq!(
            Type::try_parse(input, Location::default()),
            Err(expected.into())
        );
    }

    #[test]
    fn parse_list() {
        let input = "i32[]";
        let expected = Type::List(Box::new(Type::I32));
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn parse_double_list() {
        let input = "i8[][]";
        let expected = Type::List(Box::new(Type::List(Box::new(Type::I8))));
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn parse_triple_list() {
        let input = "i16[][][]";
        let expected = Type::List(Box::new(Type::List(Box::new(Type::List(Box::new(
            Type::I16,
        ))))));
        assert_eq!(Type::try_parse(input, Location::default()), Ok(expected));
    }

    #[test]
    fn parse_list_unclosed_returns_err() {
        let input = "int[";
        let expected = "Unclosed list".to_string();
        assert_eq!(
            Type::try_parse(input, Location::default()),
            Err(expected.into())
        );
    }

    #[test]
    fn parse_double_list_unclosed_returns_err() {
        let input = "int[][";
        let expected = "Unclosed list".to_string();
        assert_eq!(
            Type::try_parse(input, Location::default()),
            Err(expected.into())
        );
    }
}
