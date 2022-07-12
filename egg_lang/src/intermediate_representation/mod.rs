/*
This intermediate representation is very simple to maximize compatibility beteween targets.
It uses 3 simple constructs from C:
- Functions
- Structs
- Modules

These will likely be extended over time the primary goal is to use a C like IR that can then be used to extend any number of languages.
It will not closely mirror the host language, but for now I want to get maximal coverage with minimal cost.

*/

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
    pub structs: Vec<Struct>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Struct {
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub name: String,
    pub t: PrimitiveType,
}

/// The various primitive types that must be supported.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PrimitiveType {
    EggBool,
    EggI32,
}
