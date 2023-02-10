use std::{process::{Command, self, Stdio}, io::{Write, self, Read}};



pub fn select(ss: Vec<String>) -> io::Result<String> {
    let mut c = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    
        for s in ss {
            c.stdin.as_mut().expect("could not get stdin").write(s.as_bytes())?;
            c.stdin.as_mut().expect("could not get stdin").write("\n".as_bytes())?;
        }

    c.wait()?;

    let mut stdout = c.stdout.unwrap();
    let mut output = String::new();
    stdout.read_to_string(&mut output)?;

    return Ok(output);
}