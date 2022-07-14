/*
This intermediate representation is very simple to maximize compatibility beteween targets.
It uses 3 simple constructs from C:
- Functions
- Structs
- Modules

These will likely be extended over time the primary goal is to use a C like IR that can then be used to extend any number of languages.
It will not closely mirror the host language, but for now I want to get maximal coverage with minimal cost.

*/

mod egg_func;
mod egg_struct;
mod property;
pub use egg_func::*;
pub use egg_struct::*;
pub use property::*;

/// The core representation of the artifact that will be built.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub artifact_type: ArtifactType,
}

/// What type of
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ArtifactType {
    Executable(Executable),
    Library(Library),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Library {}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Executable {
    pub file_name: String,
    pub main_module: Module,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Module {
    pub file_name: String,
    pub funcs: Vec<Func>,
    pub includes: Vec<String>,
    pub structs: Vec<Struct>,
}

/// The various primitive types that must be supported.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PrimitiveType {
    EggBool,
    EggI32,
}
impl PrimitiveType {
    pub fn to_string(&self) -> String {
        match self {
            PrimitiveType::EggBool => "EggBool",
            PrimitiveType::EggI32 => "EggI32",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_type_to_str() {
        assert_eq!("EggBool", PrimitiveType::EggBool.to_string())
    }
}
