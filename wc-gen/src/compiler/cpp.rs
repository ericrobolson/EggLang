use super::compile_identifier;
use crate::{
    definition::{field::Field, function::Function, struct_::Struct, type_::Type, FromLisp},
    env::Environment,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
enum TypeReference {
    Reference(Type),
    ConstReference(Type),
    Pointer(Type),
    Value(Type),
}

pub fn compile(output_folder: PathBuf, env: &Environment) {
    let mut structs = env
        .structs
        .iter()
        .map(|(_, s)| s.clone())
        .collect::<Vec<_>>();

    // Sort structs by dependencies
    structs.sort_by(|a, b| {
        let a_referenced_types = a.get_referenced_structs();
        if a_referenced_types.contains(&b.name) {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Less;
        }
    });
    let structs = structs;

    // Add classes
    for s in structs.iter() {
        let class = compile_cpp_class(env, s);

        // Create directory
        let class_folder = output_folder.join(compile_identifier(&s.name));
        std::fs::create_dir_all(&class_folder).unwrap();

        let hpp_name = format!("{}.hpp", compile_identifier(&s.name));
        let cpp_name = {
            let name = format!("{}_generated_impl.cpp", compile_identifier(&s.name));
            class_folder.join(name)
        };
        let custom_cpp_file = {
            let name = format!("{}_custom_impl.cpp", compile_identifier(&s.name));
            class_folder.join(name)
        };

        let mut includes = vec![];
        for (_, ty) in s.get_related_types() {
            includes.append(&mut get_type_includes(&ty));
        }
        includes.sort();
        includes.dedup();

        let includes = includes
            .iter()
            .map(|i| i.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let autogen_disclaimer =
            format!("// This file was generated by wc-gen. Do not modify this file manually.\n");
        let hpp_code = format!(
            "#pragma once\n{}{}\n\n{}",
            autogen_disclaimer, includes, class.header_definition
        );
        let cpp_code = format!(
            "{}\n#include \"../{}\"\n\n{}",
            autogen_disclaimer, hpp_name, class.implementation
        );

        // Remove old files
        let hpp_path = output_folder.join(&hpp_name);
        let cpp_path = cpp_name;

        if hpp_path.exists() {
            std::fs::remove_file(&hpp_path).unwrap();
        }

        if cpp_path.exists() {
            std::fs::remove_file(&cpp_path).unwrap();
        }

        std::fs::write(hpp_path, hpp_code).unwrap();

        std::fs::write(cpp_path, cpp_code).unwrap();

        // Write custom methods and preserve existing definitions.
        let custom_cpp_path = custom_cpp_file;
        if !custom_cpp_path.exists() {
            let cpp_code = format!("#include \"../{}\"\n\n", hpp_name);
            std::fs::write(&custom_cpp_path, &cpp_code).unwrap();
        }

        // Reload code, then if the method doesn't exist, add it.
        let mut custom_code = std::fs::read_to_string(&custom_cpp_path).unwrap();
        for method in class.custom_methods.iter() {
            if !custom_code.contains(&method.function_definition) {
                custom_code.push_str(&method.implementation);
            }
        }
        std::fs::write(&custom_cpp_path, &custom_code).unwrap();
    }

    // Add function forward declarations
    let functions = env
        .functions
        .iter()
        .map(|(_, f)| f.clone())
        .collect::<Vec<_>>();

    for f in functions.iter() {
        let function = compile_cpp_function(f);
        let hpp_name = format!("{}.hpp", compile_identifier(&f.name));
        let cpp_name = format!("{}.cpp", compile_identifier(&f.name));

        let mut includes = vec![];
        for (_, ty) in f.get_related_types() {
            includes.append(&mut get_type_includes(&ty));
        }
        includes.sort();
        includes.dedup();

        let includes = includes
            .iter()
            .map(|i| i.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let hpp_code = format!(
            "#pragma once\n{}\n\n{}",
            includes, function.header_declaration
        );
        let cpp_code = format!("#include \"{}\"\n\n{}", hpp_name, function.implementation);

        // Remove old files
        let hpp_path = output_folder.join(&hpp_name);
        let cpp_path = output_folder.join(&cpp_name);

        if hpp_path.exists() {
            std::fs::remove_file(&hpp_path).unwrap();
        }

        if cpp_path.exists() {
            std::fs::remove_file(&cpp_path).unwrap();
        }

        std::fs::write(hpp_path, hpp_code).unwrap();
        std::fs::write(cpp_path, cpp_code).unwrap();
    }
}

struct Class {
    pub header_definition: String,
    pub implementation: String,
    pub custom_methods: Vec<CustomMethod>,
}
struct CustomMethod {
    pub function_definition: String,
    pub implementation: String,
}

struct CppFunction {
    pub header_declaration: String,
    pub implementation: String,
}

struct ClassMethod {
    pub header_declaration: String,
    pub implementation: String,
}

fn compile_cpp_class(env: &Environment, s: &Struct) -> Class {
    let mut header_definition = String::new();
    let mut implementation = String::new();

    header_definition.push_str(&format!("class {} \n{{\n", compile_identifier(&s.name)));
    header_definition.push_str("public:\n");
    for (name, field) in s.fields.iter() {
        let ty = map_struct_field_type(field);

        header_definition.push_str(&format!(
            "\t{} {};\n",
            compile_type(ty),
            compile_identifier(name)
        ));
    }

    // Generate functions
    let functions = vec![
        generate_constructor,
        generate_copy_constructor,
        generate_destructor,
        generate_copy_to,
        generate_clone,
        generate_equality_operator,
        generate_inequality_operator,
        generate_assignment_operator,
    ];
    for f in functions.iter() {
        let method = f(s);
        header_definition.push_str(&method.header_declaration);
        implementation.push_str(&method.implementation);
    }

    let mut custom_methods = vec![];
    for (_name, f) in s.functions.iter() {
        let method = generate_struct_fn(s, f);
        header_definition.push_str(&method.header_declaration);

        // Split off first line as we'll use that for checking if it exists
        let function_definition = method.implementation.lines().next().unwrap().to_string();
        custom_methods.push(CustomMethod {
            function_definition: format_code(&function_definition),
            implementation: format_code(&method.implementation),
        });
        // TODO: check if existing implementation exists. If so, use that.
        // implementation.push_str("\n/*\n");
        // implementation.push_str(&method.implementation);
        // implementation.push_str("*/\n");
    }

    header_definition.push_str("};\n");

    Class {
        header_definition: format_code(&header_definition),
        implementation: format_code(&implementation),
        custom_methods,
    }
}

fn format_code(code: &str) -> String {
    code.replace("):", ") : ")
        .replace("&&", "ANDAND")
        .replace("& ", " &")
        .replace("* ", " *")
        .replace("ANDAND", "&&")
}

fn generate_struct_fn(s: &Struct, f: &Function) -> ClassMethod {
    let class_name = compile_identifier(&s.name);
    let function_name = compile_identifier(&f.name);

    let code = format!("\t// TODO: Implement function\n");
    let return_type: Option<TypeReference> = match f.return_type.1 {
        Type::Void => None,
        _ => Some(TypeReference::Value(f.return_type.1.clone())),
    };

    let parameters = f
        .parameters
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                if p.type_.is_identifier() {
                    TypeReference::Reference(p.type_.clone())
                } else {
                    TypeReference::Value(p.type_.clone())
                },
            )
        })
        .collect::<Vec<_>>();

    generate_class_method(
        &function_name,
        &class_name,
        parameters,
        return_type,
        &code,
        false,
        false,
        false,
    )
}

fn generate_constructor(s: &Struct) -> ClassMethod {
    let mut constructor_code = s
        .fields
        .iter()
        .map(|(name, field)| {
            format!(
                "\t{} = {};",
                compile_identifier(name),
                init_type_value(map_struct_field_type(field))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    constructor_code.push_str("\n");
    let constructor = generate_class_method(
        &s.name,
        &s.name,
        vec![],
        None,
        &constructor_code,
        false,
        false,
        false,
    );
    constructor
}

fn generate_destructor(s: &Struct) -> ClassMethod {
    let mut destructor_code = String::new();
    s.fields
        .iter()
        .map(|(_, field)| (field.name.clone(), map_struct_field_type(field)))
        .filter_map(|(name, ty)| match ty {
            TypeReference::Pointer(_) => Some(format!("delete {};", compile_identifier(&name))),
            _ => None,
        })
        .for_each(|code| {
            destructor_code.push_str(&format!("\t{}\n", code));
        });

    let destructor = generate_class_method(
        &format!("~{}", s.name),
        &s.name,
        vec![],
        None,
        &destructor_code,
        false,
        false,
        false,
    );
    destructor
}

fn generate_copy_to(s: &Struct) -> ClassMethod {
    let mut code = String::new();

    // Copy fields
    for (_, field) in s.fields.iter() {
        let ty = map_struct_field_type(field);
        match ty {
            TypeReference::Pointer(_) => {
                // Call copy_to on the pointer
                let id = compile_identifier(&field.name);
                code.push_str(&format!("\t{}->copy_to(*other.{});\n", id, id));
            }
            _ => {
                let id = compile_identifier(&field.name);
                code.push_str(&format!("\tother.{} = {};\n", id, id));
            }
        }
    }

    generate_class_method(
        "copy_to",
        &s.name,
        vec![(
            "other".to_string(),
            TypeReference::Reference(Type::Identifier(s.name.clone())),
        )],
        Some(TypeReference::Value(Type::Void)),
        &code,
        true,
        false,
        false,
    )
}

fn generate_copy_constructor(s: &Struct) -> ClassMethod {
    let mut code = String::new();
    code.push_str("\tother.copy_to(*this);\n");

    generate_class_method(
        &s.name,
        &s.name,
        vec![(
            "other".to_string(),
            TypeReference::ConstReference(Type::Identifier(s.name.clone())),
        )],
        None,
        &code,
        false,
        false,
        true,
    )
}

fn generate_clone(s: &Struct) -> ClassMethod {
    let mut code = String::new();
    let id = compile_identifier(&s.name);
    code.push_str(&format!("\t{} clone;\n", id));
    code.push_str("\tcopy_to(clone);\n");
    code.push_str("\treturn clone;\n");

    generate_class_method(
        "clone",
        &s.name,
        vec![],
        Some(TypeReference::Value(Type::Identifier(s.name.clone()))),
        &code,
        true,
        false,
        false,
    )
}

fn generate_equality_operator(s: &Struct) -> ClassMethod {
    generate_class_method(
        "operator==",
        &s.name,
        vec![(
            "other".to_string(),
            TypeReference::ConstReference(Type::Identifier(s.name.clone())),
        )],
        Some(TypeReference::Value(Type::Bool)),
        &format!(
            "\treturn {};\n",
            s.fields
                .iter()
                .map(|(name, f)| {
                    //
                    let mapped = map_struct_field_type(f);
                    let prefix = if let TypeReference::Pointer(_) = mapped {
                        "*"
                    } else {
                        ""
                    };
                    format!(
                        "{prefix}{} == {prefix}other.{}",
                        compile_identifier(name),
                        compile_identifier(name)
                    )
                })
                .collect::<Vec<_>>()
                .join(" && ")
        ),
        true,
        false,
        false,
    )
}

fn generate_inequality_operator(s: &Struct) -> ClassMethod {
    generate_class_method(
        "operator!=",
        &s.name,
        vec![(
            "other".to_string(),
            TypeReference::ConstReference(Type::Identifier(s.name.clone())),
        )],
        Some(TypeReference::Value(Type::Bool)),
        "\treturn !(this == &other);\n",
        true,
        false,
        false,
    )
}

fn generate_assignment_operator(s: &Struct) -> ClassMethod {
    let mut code = String::new();
    code.push_str("\tif (this == &other)\n\t{\n\t\treturn *this;\n\t}\n");
    code.push_str("\tother.copy_to(*this);\n");
    code.push_str("\treturn *this;\n");

    generate_class_method(
        "operator=",
        &s.name,
        vec![(
            "other".to_string(),
            TypeReference::ConstReference(Type::Identifier(s.name.clone())),
        )],
        Some(TypeReference::Reference(Type::Identifier(s.name.clone()))),
        &code,
        false,
        false,
        false,
    )
}

fn generate_class_method(
    name: &str,
    class: &str,
    parameters: Vec<(String, TypeReference)>,
    return_type: Option<TypeReference>,
    code: &str,
    is_const: bool,
    inline: bool,
    add_constructor: bool,
) -> ClassMethod {
    let name = compile_identifier(&name);
    let class = compile_identifier(&class);

    let mut implementation = String::new();
    let mut header_declaration = String::new();

    let inline = if inline { "inline " } else { "" };
    let return_type = match return_type {
        Some(t) => format!("{} ", compile_type(t)),
        None => "".to_string(),
    };

    let mut params = String::new();
    params.push_str("(");
    params.push_str(
        &parameters
            .iter()
            .map(|(id, ty)| format!("{} {}", compile_type(ty.clone()), compile_identifier(id)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    params.push_str(")");

    header_declaration.push_str(&format!("\t{}{}", return_type, name));
    header_declaration.push_str(&params);
    if is_const {
        header_declaration.push_str(" const");
    }
    header_declaration.push_str(";\n");

    implementation.push_str(&format!("{}{}{class}::{}", inline, return_type, name));

    implementation.push_str(&params);
    if is_const {
        implementation.push_str(" const");
    }
    if add_constructor {
        implementation.push_str(&format!(":{}()", name));
    }
    implementation.push_str("\n{\n");
    implementation.push_str(code);
    implementation.push_str("}\n");

    ClassMethod {
        header_declaration,
        implementation,
    }
}

fn compile_cpp_function(f: &Function) -> CppFunction {
    let mut header_declaration = String::new();
    let mut implementation = String::new();

    header_declaration.push_str(&format!(
        "{} {}({});",
        compile_cpp_type(&f.return_type.1),
        compile_identifier(&f.name),
        f.parameters
            .iter()
            .map(|param| format!(
                "{} {}",
                compile_cpp_type(&param.type_),
                compile_identifier(&param.name)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    implementation.push_str(&format!(
        "{} {}({})\n{{\n",
        compile_cpp_type(&f.return_type.1),
        compile_identifier(&f.name),
        f.parameters
            .iter()
            .map(|param| format!(
                "{} {}",
                compile_cpp_type(&param.type_),
                compile_identifier(&param.name)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    implementation.push_str("\t// TODO: Implement function\n");

    let return_value = get_type_default(&f.return_type.1);
    if f.return_type.1 != Type::Void {
        implementation.push_str(&format!("\treturn {};\n", return_value));
    }
    implementation.push_str("}\n");

    CppFunction {
        header_declaration,
        implementation,
    }
}

fn compile_type(value: TypeReference) -> String {
    match value {
        TypeReference::Reference(t) => format!("{}&", compile_cpp_type(&t)),
        TypeReference::ConstReference(t) => format!("const {}&", compile_cpp_type(&t)),
        TypeReference::Pointer(t) => format!("{}*", compile_cpp_type(&t)),
        TypeReference::Value(t) => compile_cpp_type(&t),
    }
}

fn map_struct_field_type(field: &Field) -> TypeReference {
    if field.type_.is_identifier() {
        TypeReference::Pointer(field.type_.clone())
    } else {
        TypeReference::Value(field.type_.clone())
    }
}

fn init_type_value(ty: TypeReference) -> String {
    match ty {
        TypeReference::Reference(_) => "TODO".to_string(),
        TypeReference::ConstReference(_) => "TODO".to_string(),
        TypeReference::Pointer(t) => format!("new {}()", compile_cpp_type(&t).replace("*", "")),
        TypeReference::Value(t) => get_type_default(&t),
    }
}

fn get_type_includes(ty: &Type) -> Vec<String> {
    let mut includes = vec![];
    match ty {
        Type::List(t) => {
            includes.push(format!("#include <vector>"));
            includes.append(&mut get_type_includes(t));
        }
        Type::Identifier(i) => {
            includes.push(format!("#include \"{}.hpp\"", compile_identifier(i)));
        }
        Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64 => {
            includes.push(format!("#include <stdint.h>"));
        }
        Type::String => {
            includes.push(format!("#include <string>"));
        }
        Type::Float | Type::Bool | Type::Void => {}
    }

    includes
}

fn get_type_default(ty: &Type) -> String {
    match ty {
        Type::I8 => "0".to_string(),
        Type::I16 => "0".to_string(),
        Type::I32 => "0".to_string(),
        Type::I64 => "0".to_string(),
        Type::U8 => "0".to_string(),
        Type::U16 => "0".to_string(),
        Type::U32 => "0".to_string(),
        Type::U64 => "0".to_string(),
        Type::Bool => "false".to_string(),
        Type::String => "std::string()".to_string(),
        Type::Float => "0.0".to_string(),
        Type::Void => "()".to_string(),
        Type::Identifier(i) => format!("{}()", compile_identifier(i)),
        Type::List(ty) => format!("std::vector<{}>()", compile_cpp_type(ty).replace("*", "")),
    }
}

fn compile_cpp_type(ty: &Type) -> String {
    match ty {
        Type::I8 => "int8_t".to_string(),
        Type::I16 => "int16_t".to_string(),
        Type::I32 => "int32_t".to_string(),
        Type::I64 => "int64_t".to_string(),
        Type::U8 => "uint8_t".to_string(),
        Type::U16 => "uint16_t".to_string(),
        Type::U32 => "uint32_t".to_string(),
        Type::U64 => "uint64_t".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "std::string".to_string(),
        Type::Float => "float".to_string(),
        Type::Void => "void".to_string(),
        Type::Identifier(i) => format!("{}", compile_identifier(i)),
        Type::List(t) => format!("std::vector<{}>", compile_cpp_type(t).replace("*", "")),
    }
}
