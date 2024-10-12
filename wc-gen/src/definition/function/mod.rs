mod parameter;
pub use parameter::*;

use super::{type_::Type, FromLisp};
use lisper::Error;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Function {
    pub location: lisper::Location,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: (lisper::Location, Type),
}
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.parameters == other.parameters
            && self.return_type.1 == other.return_type.1
    }
}
impl FromLisp for Function {
    fn identifier() -> &'static str {
        "fn"
    }

    fn parse_values(list: &mut lisper::List) -> Result<Self, Error> {
        let (name, location) = list.pop_identifier("function identifier")?;

        // Parse parameters
        let mut parameters = vec![];
        let mut parameter_def = list.pop_list("list for parameters")?;
        while parameter_def.is_empty() == false {
            let mut parameter_list = parameter_def.pop_list("parameter list")?;
            let (param_type, loc) = parameter_list.pop_identifier("parameter type")?;
            let (param_name, _) = parameter_list.pop_identifier("parameter name")?;
            let param_type: Type = Type::try_parse(&param_type, loc.clone())?;

            let parameter = Parameter {
                location: loc,
                name: param_name,
                type_: param_type,
            };

            parameters.push(parameter);
        }

        // Ensure params aren't duplicated
        let mut param_names = HashSet::new();
        for parameter in &parameters {
            if param_names.contains(&parameter.name) {
                return Err(Error {
                    message: format!("Duplicate parameter '{}'", parameter.name),
                    location: parameter.location.clone(),
                });
            }
            param_names.insert(parameter.name.clone());
        }

        // Parse return type
        let (return_type, loc) = list.pop_identifier("valid return type")?;
        let return_type = Type::try_parse(&return_type, loc.clone())?;
        let return_type = (loc, return_type);

        if !list.is_empty() {
            let n = list.pop_front("Unexpected values")?;
            return Err(Error {
                message: "Expected end of list".into(),
                location: n.first_location(),
            });
        }
        let function = Function {
            location,
            name,
            parameters,
            return_type,
        };

        Ok(function)
    }

    fn get_related_types(&self) -> Vec<(lisper::Location, Type)> {
        let mut types = vec![];

        types.push((self.return_type.0.clone(), self.return_type.1.clone()));

        for parameter in &self.parameters {
            types.push((parameter.location.clone(), parameter.type_.clone()));
        }

        types
    }
}

#[cfg(test)]
mod tests {
    use lisper::Location;

    use super::*;

    fn parse(input: &str) -> lisper::List {
        lisper::parse_str(input).unwrap()[0].clone()
    }

    fn parse_fn(input: &str) -> Result<Function, Error> {
        Function::from_lisp(parse(input))
    }

    #[test]
    fn can_try_returns_false() {
        let input = "(funco life () i64)";
        let list = parse(input);
        assert_eq!(false, Function::can_try(&list));
    }

    #[test]
    fn can_try_returns_true() {
        let input = "(fn life () i64)";
        let list = parse(input);
        assert_eq!(true, Function::can_try(&list));
    }

    #[test]
    fn from_lisp_invalid_identifier_returns_err() {
        let input = "(fn 1234 () i64)";
        let result = parse_fn(input);
        let expected = "Expected function identifier".into();
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn from_lisp_invalid_param_type_returns_err() {
        let input = "(fn life true i64)";
        let result = parse_fn(input);
        let expected = "Expected list for parameters".into();
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn from_lisp_invalid_param_shape_returns_err() {
        let input = "(fn life (true) i64)";
        let result = parse_fn(input);
        let expected = "Expected parameter list".into();
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn from_lisp_duplicate_param_name_returns_err() {
        let input = "(fn life ((i64 a) (i64 a)) i64)";
        let result = parse_fn(input);
        let expected = "Duplicate parameter 'a'".into();
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn from_lisp_invalid_return_type_returns_err() {
        let input = "(fn life () true 42)";
        let result = parse_fn(input);
        let expected = "Expected valid return type".into();
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn from_lisp_simple_function_no_args_returns_ok() {
        let input = "(fn life-meaning () i64)";
        let result = parse_fn(input);

        let expected = Function {
            location: lisper::Location::default(),
            name: "life-meaning".into(),
            parameters: vec![],
            return_type: (Location::default(), Type::I64),
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn get_related_types_for_simple_function_no_args_returns_expected() {
        let input = "(fn life-meaning () i64)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::I64];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn from_lisp_simple_function_w_args_returns_ok() {
        let input = "(fn life-meaning ((i64 a) (i64 b)) i64)";
        let result = parse_fn(input);

        let expected = Function {
            location: lisper::Location::default(),
            name: "life-meaning".into(),
            parameters: vec![
                Parameter {
                    location: lisper::Location::default(),
                    name: "a".into(),
                    type_: Type::I64,
                },
                Parameter {
                    location: lisper::Location::default(),
                    name: "b".into(),
                    type_: Type::I64,
                },
            ],
            return_type: (Location::default(), Type::I64),
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn get_related_types_for_simple_function_w_args_returns_expected() {
        let input = "(fn life-meaning ((i64[] a) (i64 b)) i64)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::List(Box::new(Type::I64)), Type::I64, Type::I64];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types_for_simple_function_bool_returns_expected() {
        let input = "(fn life-meaning ((i64[] a) (i64 b)) bool)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::List(Box::new(Type::I64)), Type::Bool, Type::I64];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types_for_simple_function_string_returns_expected() {
        let input = "(fn life-meaning () string)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::String];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types_for_simple_function_float_returns_expected() {
        let input = "(fn life-meaning () f32)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::Float];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn get_related_types_for_identifier_returns_expected() {
        let input = "(fn life-meaning ((Point a)) i64)";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::Identifier("Point".into()), Type::I64];
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_returns_expr() {
        let input = "(fn life-meaning () void )";
        let result = parse_fn(input).unwrap().get_related_types();
        let mut result = result.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![Type::Void];
        expected.sort();
        assert_eq!(result, expected);
    }
}
