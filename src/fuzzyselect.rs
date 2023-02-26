use std::{
    collections::HashMap,
    io::{self, Read, Write},
    ops::Deref,
    process::{Command, Stdio},
};

// TODO: can this work on &T to avoid the need to clone?
pub fn select<'a, T>(ss: &'a HashMap<String, &T>) -> io::Result<&'a T>
where
    T: Clone,
{
    let mut subprocess = Command::new("fzf2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| {
            eprintln!("could not spawn fzf, is it installed? https://github.com/junegunn/fzf");
            std::process::exit(1);
        });

    let stdin = subprocess.stdin.as_mut().unwrap();
    for k in ss.keys() {
        stdin.write_all(k.as_bytes())?;
        stdin.write_all("\n".as_bytes())?;
    }

    // TODO: check exit code in case the user did a ctrl-C
    subprocess.wait()?;

    let mut stdout = subprocess.stdout.unwrap();
    let mut selected_key = String::new();
    stdout.read_to_string(&mut selected_key)?;

    selected_key.truncate(selected_key.trim().len());

    let value = ss.get(&selected_key).unwrap().deref();
    Ok(value)
}
