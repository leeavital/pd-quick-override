use chrono::{DateTime, TimeZone};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::{io, vec, fmt::{Display, Write}};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct UserResponse {
    pub users: Vec<User>,
    more: bool,
    limit: i32,
    offset: i32,
}

#[derive(Deserialize, Debug)]
struct MeResponse {
    user: User,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.email.as_str())?;
        f.write_str(" (")?;
        f.write_str(self.id.as_str())?;
        f.write_str(")")?;
        return Ok(());
    }
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct SchedulesResponse {
    pub schedules: Vec<Schedule>,
    pub more: bool,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Schedule {
    pub id: String,
    pub name: String,
}

impl  Display for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())?;
        f.write_str(" (")?;
        f.write_str(self.id.as_str())?;
        return f.write_char(')');
    }
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

        let page_size = 100;

        let mut all_users = Vec::new();
        loop {
            let req = client
                .get("https://api.pagerduty.com/users")
                .query(&[("offset", offset), ("limit", page_size)]);

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
        let client = reqwest::Client::new();

        let mut all_schedules = Vec::new();
        let mut offset  = 0;
        let page_size = 100;
        loop {
            let req = client.get("https://api.pagerduty.com/schedules")
                .query(&[("offset", offset), ("limit", page_size)]);
            let resp = self.add_common_headers(req).send().await?;
            let schedules = resp.json::<SchedulesResponse>().await?;
            offset += schedules.limit;
            all_schedules.extend(schedules.schedules);

            if !schedules.more {
                return Ok(all_schedules);
            } 
        }
    }

    pub async fn get_me(&self) -> reqwest::Result<User> {
        let client = reqwest::Client::new();
        let req = client.get("https://api.pagerduty.com/users/me");
        let resp = self.add_common_headers(req).send().await?;
        let user = resp.json::<MeResponse>().await?;

        return Ok(user.user);


    }

    pub async fn create_schedule_override<Tz, O>(&self, u: &User, s: &Schedule, from: DateTime<Tz>, to: DateTime<Tz>) -> reqwest::Result<()>
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
                        id: u.id.clone(),
                        r#type: "user_reference".to_string(),
                    },
                },
            ],
        };

        let client = reqwest::Client::new();
        let req = client.post(format!("https://api.pagerduty.com/schedules/{}/overrides", s.id));
        let r2 = self.add_common_headers(req).json(&override_request);

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
