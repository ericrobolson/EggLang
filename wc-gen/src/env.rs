use crate::definition::{
    enum_::Enum, function::Function, output::Output, struct_::Struct, type_::Type, FromLisp,
};
use lisper::{Error, List};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub structs: HashMap<String, Struct>,
    pub enums: HashMap<String, Enum>,
    pub functions: HashMap<String, Function>,
    pub outputs: Vec<Output>,
}
impl Environment {
    fn validate_identifier_types(&self) -> Result<(), Error> {
        for (_, value) in self.structs.iter() {
            for (location, ty) in value.get_related_types() {
                if let Type::Identifier(ty) = ty.inner_type() {
                    if self.structs.contains_key(&ty.to_string()) == false
                        && self.enums.contains_key(&ty.to_string()) == false
                    {
                        return Err(Error {
                            message: format!("Unknown type '{}'", ty),
                            location,
                        });
                    }
                }
            }
        }

        for (_, value) in self.enums.iter() {
            for (location, ty) in value.get_related_types() {
                if let Type::Identifier(ty) = ty.inner_type() {
                    if self.structs.contains_key(&ty.to_string()) == false
                        && self.enums.contains_key(&ty.to_string()) == false
                    {
                        return Err(Error {
                            message: format!("Unknown type '{}'", ty),
                            location,
                        });
                    }
                }
            }
        }

        for (_, value) in self.functions.iter() {
            for (location, ty) in value.get_related_types() {
                if let Type::Identifier(ty) = ty.inner_type() {
                    if self.structs.contains_key(&ty.to_string()) == false
                        && self.enums.contains_key(&ty.to_string()) == false
                    {
                        return Err(Error {
                            message: format!("Unknown type '{}'", ty),
                            location,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_no_duplicate_names(&self) -> Result<(), Error> {
        // Validate structs against other things
        for (name, value) in self.structs.iter() {
            if self.enums.contains_key(name) {
                return Err(Error {
                    message: format!("Struct '{}' has the same name as a enum", name),
                    location: value.location.clone(),
                });
            }

            if self.functions.contains_key(name) {
                return Err(Error {
                    message: format!("Struct '{}' has the same name as a function", name),
                    location: value.location.clone(),
                });
            }
        }

        // Validate enums against other things
        for (name, value) in self.enums.iter() {
            if self.structs.contains_key(name) {
                return Err(Error {
                    message: format!("Enum '{}' has the same name as a struct", name),
                    location: value.location.clone(),
                });
            }

            if self.functions.contains_key(name) {
                return Err(Error {
                    message: format!("Enum '{}' has the same name as a function", name),
                    location: value.location.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn parse(lists: Vec<List>) -> Result<Self, Error> {
        parse(lists)
    }
}

fn parse(lists: Vec<List>) -> Result<Environment, Error> {
    let mut structs = HashMap::new();
    let mut enums = HashMap::new();
    let mut functions = HashMap::new();
    let mut outputs = vec![];

    for list in lists {
        let definition = parse_list(list)?;
        match definition {
            Definition::Empty => {}
            Definition::Output(output) => outputs.push(output),
            Definition::Enum(enum_) => match enums.insert(enum_.name.clone(), enum_.clone()) {
                Some(_) => {
                    return Err(lisper::Error {
                        message: format!("Duplicate enum '{}'", enum_.name),
                        location: enum_.location,
                    })
                }
                None => {}
            },
            Definition::Struct(struct_) => {
                match structs.insert(struct_.name.clone(), struct_.clone()) {
                    Some(_) => {
                        return Err(lisper::Error {
                            message: format!("Duplicate struct '{}'", struct_.name),
                            location: struct_.location,
                        })
                    }
                    None => {}
                }
            }
            Definition::Function(function) => {
                match functions.insert(function.name.clone(), function.clone()) {
                    Some(_) => {
                        return Err(lisper::Error {
                            message: format!("Duplicate function '{}'", function.name),
                            location: function.location,
                        })
                    }
                    None => {}
                }
            }
        }
    }

    let env = Environment {
        structs,
        enums,
        outputs,
        functions,
    };

    env.validate_no_duplicate_names()?;
    env.validate_identifier_types()?;

    Ok(env)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Empty,
    Enum(Enum),
    Struct(Struct),
    Function(Function),
    Output(Output),
}

fn parse_list(list: List) -> Result<Definition, Error> {
    if list.is_empty() {
        return Ok(Definition::Empty);
    }

    if Struct::can_try(&list) {
        let s = Struct::from_lisp(list)?;
        return Ok(Definition::Struct(s));
    }

    if Enum::can_try(&list) {
        let e = Enum::from_lisp(list)?;
        return Ok(Definition::Enum(e));
    }

    if Function::can_try(&list) {
        let f = Function::from_lisp(list)?;
        return Ok(Definition::Function(f));
    }

    if Output::can_try(&list) {
        let o = Output::from_lisp(list)?;
        return Ok(Definition::Output(o));
    }

    let msg = format!("Expected enum, fn or struct, got '{}'", list);

    Err(Error {
        message: msg,
        location: list.location(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use lisper::Location;

    fn make(contents: &str) -> List {
        lisper::parse_str(contents)
            .unwrap()
            .first()
            .unwrap()
            .clone()
    }

    #[test]
    fn parse_returns_environment() {
        let lists = lisper::parse_str("(struct foo)").unwrap();
        let expected = Ok(Environment {
            enums: HashMap::new(),
            functions: HashMap::new(),
            outputs: vec![],
            structs: vec![(
                "foo".to_string(),
                Struct {
                    location: Location::default(),
                    name: "foo".to_string(),
                    fields: Default::default(),
                    functions: Default::default(),
                },
            )]
            .into_iter()
            .collect(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_duplicate_struct_returns_err() {
        let list = make("(struct foo)");

        let lists = vec![list.clone(), list];

        let expected = Err(Error {
            message: "Duplicate struct 'foo'".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_list_empty_returns_empty() {
        let list = make("()");
        let expected = Ok(Definition::Empty);
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn parse_list_returns_output() {
        let list = make("(output cpp ../output output.hpp)");
        let expected = Ok(Definition::Output(Output {
            location: Location::default(),
            folder: std::path::PathBuf::from("../output"),
            language: crate::definition::output::TargetLanguage::Cpp,
        }));
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn parse_list_struct_returns_struct() {
        let list = make("(struct foo)");
        let expected = Ok(Definition::Struct(Struct {
            location: Location::default(),
            name: "foo".to_string(),
            fields: Default::default(),
            functions: Default::default(),
        }));
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn parse_list_enum_returns_enum() {
        let list = make("(enum foo)");
        let expected = Ok(Definition::Enum(Enum {
            location: Location::default(),
            name: "foo".to_string(),
            variants: Default::default(),
        }));
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn parse_function_returns_expected() {
        let list = make("(fn life () i64)");
        let expected = Ok(Definition::Function(Function {
            location: Location::default(),
            name: "life".to_string(),
            parameters: vec![],
            return_type: (Location::default(), Type::I64),
        }));
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn parse_list_unknown_returns_err() {
        let list = make("(foo)");
        let expected = Err(Error {
            message: "Expected enum, fn or struct, got '(foo)'".to_string(),
            location: Location::default(),
        });
        assert_eq!(parse_list(list), expected);
    }

    #[test]
    fn struct_has_unknown_type_returns_err() {
        let lists = lisper::parse_str("(struct foo (fields (jaja bar)))").unwrap();
        let expected = Err(Error {
            message: "Unknown type 'jaja'".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    fn enum_has_unknown_type_returns_err() {
        let lists = lisper::parse_str("(enum foo Point (Pointz Point))").unwrap();
        let expected = Err(Error {
            message: "Unknown type 'Point'".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore]
    fn function_has_unknown_type_returns_err() {
        let lists = lisper::parse_str("(fn life () Point point)").unwrap();
        let expected = Err(Error {
            message: "Unknown type 'Point'".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    fn duplicate_function_returns_err() {
        let lists = lisper::parse_str("(fn life () i64)\n(fn life () i64)").unwrap();
        let expected = Err(Error {
            message: "Duplicate function 'life'".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    #[test]
    fn function_references_struct_returns_ok() {
        let input = "
        (struct Point (fields (i64 x) (i64 y)))
        (fn life ((Point a)) i64)";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn function_references_enum_returns_ok() {
        let input = "
        (enum Point)
        (fn life ((Point a)) i64)";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn struct_references_struct_returns_ok() {
        let input = "
        (struct Baz (fields (foo bar)))
        (struct foo 
            (fields 
                (foo bar)))";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn struct_references_enum_returns_ok() {
        let input = "
        (enum value)
        (struct foo 
            (fields 
                (value bar)))";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn enum_references_enum_returns_ok() {
        let input = "
        (enum Testy (value value))
        (enum value)";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn enum_references_struct_returns_ok() {
        let input = "
        (enum Testy (value value))
        (struct value)";

        let lists = lisper::parse_str(input).unwrap();

        let result = parse(lists);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn struct_has_same_name_as_enum_returns_err() {
        let lists = lisper::parse_str("(struct foo)\n(enum foo)").unwrap();
        let expected = Err(Error {
            message: "Struct 'foo' has the same name as a enum".to_string(),
            location: Location::default(),
        });
        let result = parse(lists);

        assert_eq!(result, expected);
    }

    // #[test]
    // fn struct_has_same_name_as_func_returns_err() {
    //     todo!()
    // }

    // #[test]
    // fn enum_has_same_name_as_func_returns_err() {
    //     todo!()
    // }
}
