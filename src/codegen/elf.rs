use std::{process::Command, env};

pub struct MachineCodeGenerator {
    clang_path: String,
}

impl MachineCodeGenerator {
    pub fn new() -> Self {
        let clang_env = if let Ok(p) = env::var("ZIGET_CLANG_PATH") {
            p
        } else {
            panic!("Clang path not set in the ZIGET_CLANG_PATH environment variable!");
        };

        Self {
            clang_path: clang_env
        }
    }

    pub fn generate_assembly_file(&self, input_ir: &str, output_asm: &str) {
        Command::new(&self.clang_path)
            .arg("-S")
            .arg(input_ir)
            .arg("-o")
            .arg(output_asm)
            .arg("-Wno-override-module")
            .status()
            .expect("Failed to generate assembly file");
    }

    pub fn generate_object_file(&self, input_asm: &str, output_obj: &str) {
        Command::new(&self.clang_path)
            .arg("-c")
            .arg(input_asm)
            .arg("-o")
            .arg(output_obj)
            .arg("-Wno-override-module")
            .status()
            .expect("Failed to generate object file");
    }

    pub fn link_executable(&self, input_obj: &str, output_exe: &str) {
        Command::new(&self.clang_path)
            .arg(input_obj)
            .arg("-o")
            .arg(output_exe)
            .arg("-pie")
            .arg("-lc")
            .status()
            .expect("Failed to link executable");
    }
}
