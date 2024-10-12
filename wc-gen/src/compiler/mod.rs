mod cpp;

use crate::{definition::output::TargetLanguage, env::Environment};

pub fn compile(env: Environment) {
    for output in env.outputs.iter() {
        match output.language {
            TargetLanguage::Cpp => {
                std::fs::create_dir_all(output.folder.clone()).unwrap();
                cpp::compile(output.folder.clone(), &env);
            }
        }
    }
}

fn compile_identifier(i: &str) -> String {
    i.replace("-", "_").replace("?", "")
}
