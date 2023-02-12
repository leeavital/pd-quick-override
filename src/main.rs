use std::collections::HashMap;

use client::Client;

mod client;
mod fuzzyselect;
mod timeparse;

#[tokio::main]
async fn main() {
    let client = Client::new().expect("could not open pagerduty client");

    let s = client.get_schedules().await.unwrap();
    println!("{:?}", s);

    if true {
        let users = client.get_users().await.expect("could not load users!");
        let mut users_by_email = HashMap::new();
        users.into_iter().for_each(|u| {
            users_by_email.insert(u.email.clone(), u);
        });

        let s = fuzzyselect::select(users_by_email).expect("could not read it");
        println!("{:?}", s);
    }
}
