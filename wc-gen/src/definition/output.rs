use super::FromLisp;
use lisper::Location;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum TargetLanguage {
    Cpp,
}

/// A target output to compile to.
#[derive(Debug, Clone)]
pub struct Output {
    pub location: Location,
    pub folder: PathBuf,
    pub language: TargetLanguage,
}
impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        self.folder == other.folder && self.language == other.language
    }
}

impl FromLisp for Output {
    fn identifier() -> &'static str {
        "output"
    }

    fn get_related_types(&self) -> Vec<(Location, super::type_::Type)> {
        vec![]
    }

    fn parse_values(list: &mut lisper::List) -> Result<Self, lisper::Error> {
        let (language, location) = list.pop_identifier("language")?;
        let language = match language.as_str() {
            "c++" | "cpp" => TargetLanguage::Cpp,
            _ => {
                return Err(lisper::Error {
                    message: format!("Unknown language '{}'", language),
                    location,
                })
            }
        };

        let (folder, location) = list.pop_identifier("folder")?;
        let folder = PathBuf::from(folder);

        Ok(Output {
            location,
            folder,
            language,
        })
    }
}

#[cfg(test)]

mod tests {
    use std::default;

    use lisper::Location;

    use super::*;

    fn parse(input: &str) -> lisper::List {
        lisper::parse_str(input).unwrap()[0].clone()
    }

    #[test]
    fn can_try_empty_returns_false() {
        let input = parse("()");
        assert_eq!(Output::can_try(&input), false);
    }

    #[test]
    fn can_try_not_identifier_returns_false() {
        let input = parse("(foo-bar)");
        assert_eq!(Output::can_try(&input), false);
    }

    #[test]
    fn can_try_identifier_returns_true() {
        let input = parse(format!("({})", Output::identifier()).as_str());
        assert_eq!(Output::can_try(&input), true);
    }

    #[test]
    fn unknown_language_returns_error() {
        let input = "(output unknown ../output)";
        let list = parse(input);
        let value = Output::from_lisp(list);
        assert!(value.is_err());
        assert_eq!(value, Err("Unknown language 'unknown'".into()))
    }

    #[test]
    fn outputs_cpp() {
        let input = "(output c++ ../output main.hpp)";
        let list = parse(input);
        let value = Output::from_lisp(list).unwrap();
        let expected = Output {
            location: Location::default(),
            folder: PathBuf::from("../output"),
            language: TargetLanguage::Cpp,
        };

        assert_eq!(value, expected);
        assert_eq!(value.get_related_types(), vec![]);
    }
}
