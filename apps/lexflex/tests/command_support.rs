use std::process::Command;

pub struct CommandOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn run(args: &[&str]) -> CommandOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args(args)
        .current_dir(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../"))
        .output()
        .expect("launch lexflex-app");

    CommandOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

pub fn run_with_stdin(args: &[&str], input: &str) -> CommandOutput {
    use std::io::Write;
    use std::process::Stdio;

    let mut command = Command::new(env!("CARGO_BIN_EXE_lexflex-app"));
    let mut child = command
        .args(args)
        .current_dir(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch lexflex-app");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for lexflex-app");
    CommandOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}
