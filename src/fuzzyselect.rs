use std::{process::{Command, self, Stdio}, io::{Write, self, Read}};



pub fn select(ss: Vec<String>) -> io::Result<String> {
    let mut subprocess = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = subprocess.stdin.as_mut().unwrap();
    for s in ss {
        stdin.write(s.as_bytes())?;
        stdin.write("\n".as_bytes())?;
    }

    subprocess.wait()?;

    let mut stdout = subprocess.stdout.unwrap();
    let mut output = String::new();
    stdout.read_to_string(&mut output)?;

    return Ok(output);
}