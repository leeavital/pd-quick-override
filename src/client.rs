use chrono::{DateTime, Utc, TimeZone};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::{io, vec, fmt::Display};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct UserResponse {
    pub users: Vec<User>,
    more: bool,
    limit: i32,
    offset: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct SchedulesResponse {
    pub schedules: Vec<Schedule>,
    pub more: bool,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Schedule {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct ScheduleOverrideRequest {
    overrides: Vec<ScheduleOverride>,
}

#[derive(Debug, Serialize)]
struct ScheduleOverride {
    start: String,
    end: String,
    user: UserRef,
}

#[derive(Debug, Serialize)]
struct UserRef {
    id: String,
    r#type: String,
}

pub struct Client {
    api_key: String,
}

impl Client {
    pub fn new() -> std::result::Result<Client, Box<dyn std::error::Error>> {
        let api_key = Self::get_api_key()?;

        return Ok(Client { api_key });
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
                return Ok(String::from(prompt.trim()));
            }
            Err(e) => return Err(Box::from(e)),
        }
    }

    pub async fn get_users(&self) -> reqwest::Result<Vec<User>> {
        let client = reqwest::Client::new();

        let mut offset = 0;

        let mut all_users = Vec::new();
        loop {
            let req = client
                .get("https://api.pagerduty.com/users")
                .query(&[("offset", offset)]);

            let resp = self.add_common_headers(req).send().await?;
            let users = resp.json::<UserResponse>().await?;
            for u in users.users {
                all_users.push(u);
            }

            offset += users.limit;
            if !users.more {
                return Ok(all_users);
            }
        }
    }

    pub async fn get_schedules(&self) -> reqwest::Result<Vec<Schedule>> {
        // TODO: pagination
        let client = reqwest::Client::new();

        let req = client.get("https://api.pagerduty.com/schedules");

        let resp = self.add_common_headers(req).send().await?;
        let schedules = resp.json::<SchedulesResponse>().await?;
        return Ok(schedules.schedules);
    }

    pub async fn create_schedule_override<Tz, O>(&self, u: User, s: Schedule, from: DateTime<Tz>, to: DateTime<Tz>) -> reqwest::Result<()>
    where 
        Tz : TimeZone<Offset = O>,
        O : Display
    {
    
        let override_request = ScheduleOverrideRequest{
            overrides: vec![
                ScheduleOverride{
                    start: from.to_rfc3339(),
                    end: to.to_rfc3339(),
                    user: UserRef {
                        id: u.id, 
                        r#type: "user_reference".to_string(),
                    },
                },
            ],
        };

        let client = reqwest::Client::new();
        let req = client.post(format!("https://api.pagerduty.com/schedules/{}/overrides", s.id));
        let r2 = self.add_common_headers(req).json(&override_request);

        println!("{:?}", r2);

        r2.send().await?;

        return  Ok(());

    }

    fn add_common_headers(&self, req: RequestBuilder) -> RequestBuilder {
        let mut api_key_value = String::from("Token token=");
        api_key_value.push_str(&self.api_key);

        return req
            .header("Authorization", api_key_value)
            .header("Accept", "application/vnd.pagerduty+json;version=2")
            .header("Content-Type", "application/json");
    }
}
