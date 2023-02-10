
mod fuzzyselect;
mod client;


#[tokio::main]
async fn main() {
    let r = vec![String::from("hello"), String::from("goodbye")];

    let u = client::get_users().await;

    println!("{:?}", u.unwrap());

    let s = fuzzyselect::select(r).expect("could not read it");
    print!("{s}");
}
