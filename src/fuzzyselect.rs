use std::{process::{Command, Stdio}, io::{Write, self, Read}, collections::HashMap};



pub fn select<T>(ss: HashMap<String, T>) -> io::Result<T>
where
    T: Clone 
{
    let mut subprocess = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = subprocess.stdin.as_mut().unwrap();
    for (k, _v) in &ss {
        stdin.write(k.as_bytes())?;
        stdin.write("\n".as_bytes())?;
    }

    // TODO: check exit code in case the user did a ctrl-C
    subprocess.wait()?;

    let mut stdout = subprocess.stdout.unwrap();
    let mut selected_key = String::new();
    stdout.read_to_string(&mut selected_key)?;

    selected_key.truncate(selected_key.trim().len());

    let value = ss.get(&selected_key).unwrap().clone();
    return Ok(value);
}