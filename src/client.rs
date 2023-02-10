use std::{fs::read_link, io};

use clap::error::Error;
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

pub struct Client {
    api_key: String,
}


impl  Client {

    pub fn new() -> std::result::Result<Client, Box<dyn std::error::Error>> {
        let api_key = Self::get_api_key()?;

        return Ok(Client{
            api_key,
        });
    }


    fn get_api_key() -> std::result::Result<String, Box<dyn std::error::Error>> {
        let keyring_entry = keyring::Entry::new("pd-fast-override", "api-key");
        match keyring_entry.get_password() {
            Ok(secret) => Ok(secret),
            Err(keyring::Error::NoEntry) => {
                println!("no entry found in keyring, enter an API key");
               
                let mut prompt = String::new();
                io::stdin().read_line(&mut prompt)?;


                keyring_entry.set_password(prompt.trim())?;

                // TODO: avoid clone
                return  Ok(String::from(prompt.trim()));
            },
            Err(e) => return Err(Box::from(e)),
            
        }


    }

    pub async fn get_users(&self) -> reqwest::Result<UserResponse> {
        let client = reqwest::Client::new();

        let mut api_key_value = String::from("Token token=");
        api_key_value.push_str(&self.api_key);

        let resp = client.get("https://api.pagerduty.com/users")
            .header("Authorization", api_key_value)
            .header("Accept",  "application/vnd.pagerduty+json;version=2")
            .header("Content-Type", "application/json")
            .send()
            .await?;


        let users = resp.json::<UserResponse>().await?;

        return Ok(users);
    }
}