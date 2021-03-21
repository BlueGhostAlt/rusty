pub const ERROR_MESSAGES: [&str; 4] = [
    "Failed to create directory.",
    "Failed to create file.",
    "Failed to write to file.",
    "Failed to execute child.",
];

pub fn extract_code_from_message(msg: &str) -> Option<&str> {
    let prefix_and_rest = msg.split_once(|ch: char| ch.is_ascii_whitespace());
    if matches!(prefix_and_rest, None) {
        return None;
    }
    let (prefix, rest) = prefix_and_rest.unwrap();

    if !prefix.eq_ignore_ascii_case("eval!") || rest.len() < 6 {
        return None;
    }

    let (end_left, lang, end_right) = (&rest[..3], &rest[3..5], &rest[rest.len() - 3..]);
    if (end_left, lang, end_right) != ("```", "rs", "```") {
        return None;
    }

    let code = &rest[5..rest.len() - 3].trim();

    Some(code)
}

pub mod eval {
    use std::{
        fmt,
        fs::{self, File},
        io::Write,
        path::Path,
        process::Command,
        str,
    };

    use super::ERROR_MESSAGES;

    const EXECUTION_COMMAND: &str = if cfg!(windows) {
        ".\\eval\\main.exe"
    } else {
        "./eval/main"
    };

    #[derive(Copy, Clone)]
    pub enum Error {
        FailedToCreateDirectory,
        FailedToCreateFile,
        FailedToWriteToFile,
        FailedToExecuteChild,
    }

    impl fmt::Debug for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "```ERROR:\n{}```", ERROR_MESSAGES[*self as usize])
        }
    }

    fn generate_code_from_expression(expr: &str) -> String {
        format!("#![feature(type_name_of_val)]\nuse std::any;\n\nfn main() {{\n    let expr = {};\n\n    println!(\"{{:?}}: {{}}\", expr, any::type_name_of_val(&expr));\n}}\n", expr)
    }

    fn create_eval_dir_if_not_exists() -> Result<(), Error> {
        if !Path::new("eval/").exists() {
            fs::create_dir("eval")
                .ok()
                .ok_or(Error::FailedToCreateDirectory)?;
        }

        Ok(())
    }

    fn write_code_to_file(code: &str) -> Result<(), Error> {
        let mut file = File::create("eval/main.rs")
            .ok()
            .ok_or(Error::FailedToCreateFile)?;

        file.write_all(generate_code_from_expression(code).as_bytes())
            .ok()
            .ok_or(Error::FailedToWriteToFile)?;

        Ok(())
    }

    fn compile_code() -> Result<Option<String>, Error> {
        let output = Command::new("rustc")
            .current_dir("eval")
            .arg("main.rs")
            .output()
            .ok()
            .ok_or(Error::FailedToExecuteChild)?;

        if !output.status.success() {
            unsafe {
                let stderr = str::from_utf8_unchecked(&output.stderr);
                let output = format!("```{}```", stderr);

                return Ok(Some(output));
            }
        }

        Ok(None)
    }

    fn run_binary() -> Result<String, Error> {
        let output = Command::new(EXECUTION_COMMAND)
            .output()
            .ok()
            .ok_or(Error::FailedToExecuteChild)?;

        unsafe {
            let stdout = str::from_utf8_unchecked(&output.stdout);
            let stderr = str::from_utf8_unchecked(&output.stderr);
            let output = format!("```{}\n{}```", stderr, stdout);

            Ok(String::from(output.trim()))
        }
    }

    pub fn execute_code(code: &str) -> Result<String, Error> {
        create_eval_dir_if_not_exists()?;
        write_code_to_file(code)?;

        if let Some(output) = compile_code()? {
            return Ok(output);
        }

        run_binary()
    }
}
