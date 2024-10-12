use super::{field::Field, function::Function, type_::Type, FromLisp};
use lisper::{Error, Location};
use std::collections::HashMap;

/// A struct definition. Contains a name and a list of fields and can be sent over the network. Not tied to anything in particular.
#[derive(Debug, Clone)]
pub struct Struct {
    pub location: lisper::Location,
    pub name: String,
    pub fields: HashMap<String, Field>,
    pub functions: HashMap<String, Function>,
}
impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.fields == other.fields
    }
}

impl Struct {
    /// Returns a list of all the other structs referenced by the struct.
    pub fn get_referenced_structs(&self) -> Vec<String> {
        self.fields
            .values()
            .filter_map(|field| match field.type_.inner_type() {
                Type::Identifier(name) => Some(name),
                _ => None,
            })
            .collect()
    }
}

impl FromLisp for Struct {
    fn identifier() -> &'static str {
        "struct"
    }

    fn parse_values(list: &mut lisper::List) -> Result<Self, Error> {
        let (name, location) = list.pop_identifier("Expected name")?;
        let mut fields = HashMap::new();
        let mut functions = HashMap::new();

        // Parse fields
        if !list.is_empty() {
            let mut list = list.pop_list("fields")?;
            let (id, loc) = list.pop_identifier("fields")?;
            if id != "fields" {
                return Err(Error {
                    message: "Expected fields".into(),
                    location: loc,
                });
            }

            while !list.is_empty() {
                Field::parse_field("property", &mut list, &mut fields)?;
            }
        }

        // Parse functions
        while !list.is_empty() {
            let function_list = list.pop_list("functions")?;
            if Function::can_try(&function_list) {
                let f = Function::from_lisp(function_list)?;
                let location = f.location.clone();
                let name = f.name.clone();
                match functions.insert(f.name.clone(), f) {
                    Some(_) => {
                        return Err(Error {
                            message: format!("Duplicate function '{}'", name),
                            location: location,
                        });
                    }
                    None => (),
                }
            } else {
                let n = list.pop_front("Unexpected values")?;
                return Err(Error {
                    message: "Expected end of list".into(),
                    location: n.first_location(),
                });
            }
        }

        // Ensure nothing else remains
        if list.is_empty() == false {
            let n = list.pop_front("Unexpected values")?;
            return Err(Error {
                message: "Expected end of list".into(),
                location: n.first_location(),
            });
        }

        Ok(Struct {
            location,
            name,
            fields,
            functions,
        })
    }

    fn get_related_types(&self) -> Vec<(Location, Type)> {
        let mut types = vec![];
        self.fields
            .values()
            .for_each(|field| types.push((field.location.clone(), field.type_.clone())));

        self.functions.values().for_each(|f| {
            let fn_types = f.get_related_types();
            fn_types
                .iter()
                .for_each(|(loc, ty)| types.push((loc.clone(), ty.clone())));
        });

        types
    }
}

#[cfg(test)]

mod tests {
    use std::default;

    use lisper::Location;

    use crate::definition::function::Parameter;

    use super::*;

    fn parse(input: &str) -> lisper::List {
        lisper::parse_str(input).unwrap()[0].clone()
    }

    #[test]
    fn can_try_empty_returns_false() {
        let input = parse("()");
        assert_eq!(Struct::can_try(&input), false);
    }

    #[test]
    fn can_try_not_identifier_returns_false() {
        let input = parse("(foo-bar)");
        assert_eq!(Struct::can_try(&input), false);
    }

    #[test]
    fn can_try_identifier_returns_true() {
        let input = parse(format!("({})", Struct::identifier()).as_str());
        assert_eq!(Struct::can_try(&input), true);
    }

    #[test]
    fn from_lisp_wrong_type_returns_err() {
        let input = parse("(f00 foo)");
        let expected = Err("Expected identifier 'struct'".into());

        assert_eq!(Struct::from_lisp(input), expected);
    }

    #[test]
    fn from_lisp_fields_returns_empty() {
        let input = parse("(struct foo)");
        let expected = Struct {
            location: Location::default(),
            name: "foo".to_string(),
            fields: HashMap::new(),
            functions: HashMap::new(),
        };

        assert_eq!(Struct::from_lisp(input), Ok(expected));
    }

    #[test]
    fn parses_function_returns_struct() {
        let input = parse(
            "
(struct Aabb
    (fields 
        (i32 x-min)
        (i32 y-min)
        (i32 x-max)
        (i32 y-max))
    (fn collides? ((Aabb other)) bool))",
        );
        let actual = Struct::from_lisp(input).unwrap();

        let expected = Struct {
            location: Location::default(),
            name: "Aabb".to_string(),
            fields: vec![
                (
                    "x-min".to_string(),
                    Field {
                        location: Location::default(),
                        name: "x-min".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "y-min".to_string(),
                    Field {
                        location: Location::default(),
                        name: "y-min".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "x-max".to_string(),
                    Field {
                        location: Location::default(),
                        name: "x-max".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "y-max".to_string(),
                    Field {
                        location: Location::default(),
                        name: "y-max".to_string(),
                        type_: Type::I32,
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            functions: vec![(
                "collides?".to_string(),
                Function {
                    location: Location::default(),
                    name: "collides?".to_string(),
                    parameters: vec![Parameter {
                        location: Location::default(),
                        name: "other".to_string(),
                        type_: Type::Identifier("Aabb".to_string()),
                    }],
                    return_type: (Location::default(), Type::Bool),
                },
            )]
            .iter()
            .cloned()
            .collect(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_two_function_returns_struct() {
        let input = parse(
            "
(struct Aabb
    (fields 
        (i32 x-min)
        (i32 y-min)
        (i32 x-max)
        (i32 y-max))
    (fn collides? ((Aabb other)) bool)
    (fn print () void))",
        );
        let actual = Struct::from_lisp(input).unwrap();

        let expected = Struct {
            location: Location::default(),
            name: "Aabb".to_string(),
            fields: vec![
                (
                    "x-min".to_string(),
                    Field {
                        location: Location::default(),
                        name: "x-min".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "y-min".to_string(),
                    Field {
                        location: Location::default(),
                        name: "y-min".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "x-max".to_string(),
                    Field {
                        location: Location::default(),
                        name: "x-max".to_string(),
                        type_: Type::I32,
                    },
                ),
                (
                    "y-max".to_string(),
                    Field {
                        location: Location::default(),
                        name: "y-max".to_string(),
                        type_: Type::I32,
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            functions: vec![
                (
                    "print".to_string(),
                    Function {
                        location: Location::default(),
                        name: "print".to_string(),
                        parameters: vec![],
                        return_type: (Location::default(), Type::Void),
                    },
                ),
                (
                    "collides?".to_string(),
                    Function {
                        location: Location::default(),
                        name: "collides?".to_string(),
                        parameters: vec![Parameter {
                            location: Location::default(),
                            name: "other".to_string(),
                            type_: Type::Identifier("Aabb".to_string()),
                        }],
                        return_type: (Location::default(), Type::Bool),
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_function_duplicate_returns_err() {
        let input = parse(
            "
(struct Aabb
    (fields 
        (i32 x-min)
        (i32 y-min)
        (i32 x-max)
        (i32 y-max))
    (fn collides? ((Aabb other)) bool)
    (fn collides? ((Aabb other)) bool))",
        );
        let actual = Struct::from_lisp(input);

        let expected = Err("Duplicate function 'collides?'".into());

        assert_eq!(actual, expected);
    }

    #[test]
    fn from_list_try_fields_returns_err() {
        let input = parse(
            "(struct foo 
        (fieldo (i64 bar)))",
        );
        let expected = Err("Expected fields".into());

        assert_eq!(Struct::from_lisp(input), expected);
    }

    #[test]
    fn from_list_try_fields_not_list_returns_err() {
        let input = parse(
            "(struct foo 
        (fields i64))",
        );
        let expected = Err("Expected property list".into());

        assert_eq!(Struct::from_lisp(input), expected);
    }

    #[test]
    fn from_list_parses_single_field() {
        let input = parse(
            "(struct foo 
        (fields (i64 bar)))",
        );

        let mut fields = HashMap::new();
        fields.insert(
            "bar".to_string(),
            Field {
                location: Location::default(),
                name: "bar".to_string(),
                type_: Type::I64,
            },
        );
        let expected = Struct {
            location: Location::default(),
            name: "foo".to_string(),
            fields,
            functions: HashMap::new(),
        };

        assert_eq!(Struct::from_lisp(input), Ok(expected));
    }

    #[test]
    fn from_list_parses_multiple_fields() {
        let input = parse(
            "(struct foo 
        (fields 
            (i64 bar)
            (bool baz)
            (string qux)
        ))",
        );

        let mut fields = HashMap::new();
        fields.insert(
            "bar".to_string(),
            Field {
                location: Location::default(),
                name: "bar".to_string(),
                type_: Type::I64,
            },
        );
        fields.insert(
            "baz".to_string(),
            Field {
                location: Location::default(),
                name: "baz".to_string(),
                type_: Type::Bool,
            },
        );
        fields.insert(
            "qux".to_string(),
            Field {
                location: Location::default(),
                name: "qux".to_string(),
                type_: Type::String,
            },
        );
        let expected = Struct {
            location: Location::default(),
            name: "foo".to_string(),
            fields,
            functions: HashMap::new(),
        };

        assert_eq!(Struct::from_lisp(input), Ok(expected));
    }

    #[test]
    fn from_list_duplicate_fields_returns_err() {
        let input = parse(
            "(struct foo 
        (fields (i64 bar) (i64 bar)))",
        );

        let expected = Err("Duplicate property 'bar'".into());

        assert_eq!(Struct::from_lisp(input), expected);
    }

    #[test]
    fn get_related_types() {
        let input = "(struct Shape 
            (fields 
                (i64 x) 
                (i64 y) 
                (i64 z) 
                (i64[] points) 
                (Point point) 
                (Point[] point2s) )
            )";
        let list = parse(input);
        let value = Struct::from_lisp(list).unwrap();
        let mut expected = vec![
            Type::I64,
            Type::I64,
            Type::I64,
            Type::List(Box::new(Type::I64)),
            Type::Identifier("Point".into()),
            Type::List(Box::new(Type::Identifier("Point".into()))),
        ]
        .into_iter()
        .collect::<Vec<_>>();

        let mut result = value
            .get_related_types()
            .iter()
            .map(|(loc, ty)| ty.clone())
            .collect::<Vec<_>>();

        expected.sort();
        result.sort();

        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types_includes_fn() {
        let input = "(struct Shape 
            (fields 
                (i64 x) 
                (i64 y) 
                (i64 z) 
                (i64[] points) 
                (Point point) 
                (Point[] point2s) )
            (fn collides? ((Shape other)) bool)
            )";
        let list = parse(input);
        let value = Struct::from_lisp(list).unwrap();
        let mut expected = vec![
            Type::I64,
            Type::I64,
            Type::I64,
            Type::Bool,
            Type::List(Box::new(Type::I64)),
            Type::Identifier("Point".into()),
            Type::List(Box::new(Type::Identifier("Point".into()))),
            Type::Identifier("Shape".into()),
        ]
        .into_iter()
        .collect::<Vec<_>>();

        let mut result = value
            .get_related_types()
            .iter()
            .map(|(loc, ty)| ty.clone())
            .collect::<Vec<_>>();

        expected.sort();
        result.sort();

        assert_eq!(result, expected);
    }
}
