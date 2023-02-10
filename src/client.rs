use serde::{Deserialize}; 

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    pub users: Vec<User>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
}


pub async fn get_users() -> reqwest::Result<UserResponse> {

    // let res = reqwest::get("https://httpbin.org/ip")
    //     .await?;

    let client = reqwest::Client::new();
    let resp = client.get("https://api.pagerduty.com/users")
        .header("Authorization", "Token token=y_NbAkKc66ryYTWUXYEu")
        .header("Accept",  "application/vnd.pagerduty+json;version=2")
        .header("Content-Type", "application/json")
        .send()
        .await?;


    let users = resp.json::<UserResponse>().await?;

    return Ok(users);
}