use lisper::Location;

use super::{type_::Type, FromLisp};
use crate::definition::field::Field;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Enum {
    pub location: lisper::Location,
    pub name: String,
    pub variants: HashMap<String, Variant>,
}
impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.variants == other.variants
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub location: lisper::Location,
    pub name: String,
    pub values: HashMap<String, Field>,
}
impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.values == other.values
    }
}

impl FromLisp for Enum {
    fn identifier() -> &'static str {
        "enum"
    }

    fn parse_values(list: &mut lisper::List) -> Result<Self, lisper::Error> {
        // Get the name of the enum
        let (name, location) = list.pop_identifier("enum name")?;

        let mut variants = HashMap::new();
        while !list.is_empty() {
            // Determine if it's a simple or complex variant
            let is_complex = list.front_is_list();

            let variant = if is_complex {
                // Pop list
                let mut list = list.pop_list("complex variant")?;
                let (variant_name, variant_location) = list.pop_identifier("variant name")?;

                let mut complex_variants = HashMap::new();
                if list.front_is_identifier() {
                    // Parse the single value
                    let (field_type, loc) = list.pop_identifier("complex variant type")?;
                    let ty = Type::try_parse(&field_type, loc.clone())?;
                    let field = Field {
                        name: "value".to_string(),
                        type_: ty,
                        location: loc,
                    };
                    complex_variants.insert("value".to_string(), field);

                    if !list.is_empty() {
                        return Err(lisper::Error {
                            message:
                                "A complex variant without named fields must only have one value"
                                    .into(),
                            location: list.location(),
                        });
                    }
                } else {
                    while !list.is_empty() {
                        Field::parse_field(
                            "complex variant property",
                            &mut list,
                            &mut complex_variants,
                        )?;
                    }
                }
                Variant {
                    location: variant_location,
                    name: variant_name.clone(),
                    values: complex_variants,
                }
            } else {
                let (variant_name, variant_location) = list.pop_identifier("variant name")?;
                Variant {
                    location: variant_location,
                    name: variant_name.clone(),
                    values: HashMap::new(),
                }
            };

            let variant_location = variant.location.clone();
            let variant_name = variant.name.clone();
            match variants.insert(variant_name.clone(), variant) {
                Some(_) => {
                    return Err(lisper::Error {
                        message: format!("Duplicate variant '{}'", variant_name),
                        location: variant_location,
                    });
                }
                None => {}
            }
        }

        Ok(Enum {
            location,
            name,
            variants,
        })
    }

    fn get_related_types(&self) -> Vec<(Location, Type)> {
        let mut related_types = vec![];
        for (_, variant) in self.variants.iter() {
            for field in variant.values.iter() {
                related_types.push((field.1.location.clone(), field.1.type_.clone()));
            }
        }
        related_types
    }
}

#[cfg(test)]
mod tests {
    use lisper::Location;

    use crate::definition::type_::Type;

    use super::*;

    fn parse(input: &str) -> lisper::List {
        lisper::parse_str(input).unwrap()[0].clone()
    }

    #[test]
    fn can_try_returns_false() {
        let input = "(funco life () int 42)";
        let list = parse(input);
        assert_eq!(false, Enum::can_try(&list));
    }

    #[test]
    fn can_try_returns_true() {
        let input = "(enum Shape Circle Square Triangle)";
        let list = parse(input);
        assert_eq!(true, Enum::can_try(&list));
    }

    #[test]
    fn from_lisp_no_identifer_returns_err() {
        let input = "(enum)";
        let list = parse(input);
        let result = Enum::from_lisp(list);
        let expected = Err("Expected enum name".into());
        assert_eq!(result, expected);
    }

    #[test]
    fn from_lisp_no_values_returns_enum() {
        let input = "(enum Shape)";
        let list = parse(input);
        let expected = Enum {
            location: Location::default(),
            name: "Shape".to_string(),
            variants: HashMap::new(),
        };
        let result = Enum::from_lisp(list);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn from_lisp_duplicate_simple_variants_returns_errors() {
        let input = "(enum Shape Circle Square Circle)";
        let list = parse(input);
        let expected = Err("Duplicate variant 'Circle'".into());
        let result = Enum::from_lisp(list);
        assert_eq!(result, expected);
    }

    #[test]
    fn from_lisp_parses_simple_variants() {
        let input = "(enum Shape Circle Square Triangle)";
        let list = parse(input);
        let expected = Enum {
            location: Location::default(),
            name: "Shape".to_string(),
            variants: vec![
                (
                    "Circle".to_string(),
                    Variant {
                        location: Location::default(),
                        name: "Circle".to_string(),
                        values: HashMap::new(),
                    },
                ),
                (
                    "Square".to_string(),
                    Variant {
                        location: Location::default(),
                        name: "Square".to_string(),
                        values: HashMap::new(),
                    },
                ),
                (
                    "Triangle".to_string(),
                    Variant {
                        location: Location::default(),
                        name: "Triangle".to_string(),
                        values: HashMap::new(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };
        let result = Enum::from_lisp(list);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn from_lisp_complex_variant_no_identifier() {
        let input = "(enum Literal (String string))";
        let expected = Enum {
            location: Location::default(),
            name: "Literal".to_string(),
            variants: vec![(
                "String".to_string(),
                Variant {
                    location: Location::default(),
                    name: "String".to_string(),
                    values: vec![(
                        "value".to_string(),
                        Field {
                            location: Location::default(),
                            name: "value".to_string(),
                            type_: Type::String,
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
            )]
            .into_iter()
            .collect(),
        };

        assert_eq!(Enum::from_lisp(parse(input)), Ok(expected));
    }

    #[test]
    fn from_lisp_complex_variant_no_identifier_returns_err_multiple_types() {
        let input = "(enum Literal (String string string))";
        let expected =
            Err("A complex variant without named fields must only have one value".into());

        assert_eq!(Enum::from_lisp(parse(input)), expected);
    }

    #[test]
    fn from_lisp_parses_complex_variant() {
        let input = "(enum Shape (Point (i64 x) (i64 y)))";
        let list = parse(input);
        let expected = Enum {
            location: Location::default(),
            name: "Shape".to_string(),
            variants: vec![(
                "Point".to_string(),
                Variant {
                    location: Location::default(),
                    name: "Point".to_string(),
                    values: vec![
                        (
                            "x".to_string(),
                            Field {
                                location: Location::default(),
                                name: "x".to_string(),
                                type_: Type::I64,
                            },
                        ),
                        (
                            "y".to_string(),
                            Field {
                                location: Location::default(),
                                name: "y".to_string(),
                                type_: Type::I64,
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            )]
            .into_iter()
            .collect(),
        };
        let result = Enum::from_lisp(list);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn from_lisp_complex_variant_duplicate_fields_returns_err() {
        let input = "(enum Shape (Point (i64 x) (i64 x)))";
        let list = parse(input);
        let expected = Err("Duplicate complex variant property 'x'".into());
        let result = Enum::from_lisp(list);
        assert_eq!(result, expected);
    }

    #[test]
    fn from_lisp_duplicate_simple_and_complex_variant_returns_err() {
        let input = "(enum Shape Point (Point (i64 x)))";
        let list = parse(input);
        let expected = Err("Duplicate variant 'Point'".into());
        let result = Enum::from_lisp(list);
        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types() {
        let input = "(enum Shape 
            Aabb 
            (Circle (i64 x)) 
            (Rectangle (i64 x) (i64 y))
            (Line i64[]) 
            (Point Point)
            (Polygon Point[]) 
            )";
        let list = parse(input);
        let value = Enum::from_lisp(list).unwrap();
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
}
