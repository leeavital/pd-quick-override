
mod fuzzyselect;

fn main() {
    println!("Hello, world!");
    let r = vec![String::from("hello"), String::from("goodbye")];

    let s = fuzzyselect::select(r).expect("could not read it");
    print!("{s}");
}
