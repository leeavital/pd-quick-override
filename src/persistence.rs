
use std::{path::{PathBuf}, error::Error, ops::Mul};

use chrono::{Utc};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use tokio::{join, io::{AsyncWriteExt, AsyncReadExt}};

use crate::client::{self, User, Schedule};



#[derive(Deserialize, Serialize, Debug)]
pub struct Serialized {
    pub users: Vec<User>,
    pub schedules: Vec<Schedule>,
    pub updated_at: i64, // in seconds
}

pub struct  Database<'a> {
    client: &'a client::Client,
    pub storage: Serialized,
}


impl <'a> Database<'a> {
    pub async fn load(client:  &'a client::Client) -> Result<Database<'a>, Box<dyn Error>> {
        let storage_dir = Self::get_storage_dir();
        if !storage_dir.exists() {
            std::fs::create_dir_all(storage_dir).expect("could not create directory");
        }

        let storage_file = Self::get_storage_file();
        let mut db = Database{
            client,
            storage: Serialized { users: Vec::new(), schedules: Vec::new(), updated_at: 0 },
        };
        if !storage_file.exists() {
            println!("doing remote load...");
            db.do_remote_load().await?;
            return Ok(db);
        } else {
            println!("loading state from file");
            db.do_file_load().await?;
        }

        Ok(db)
    }

    async fn do_remote_load(&mut self) -> Result<(), Box<dyn Error>> {

        let progress = MultiProgress::new();
        let users_progress = progress.add(ProgressBar::new(100))
            .with_prefix("loading users")
            .with_style(ProgressStyle::with_template("Loading Users:     [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap());

        let schedule_progress = progress.add(ProgressBar::new(100))
            .with_prefix("loading schedules")
            .with_style(ProgressStyle::with_template("Loading Schedules: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap());


        let users = self.client.get_users(users_progress);
        let schedules = self.client.get_schedules(schedule_progress);

        let (r_users, r_schedules) = join!(users, schedules);

        let users = r_users?;
        let schedules = r_schedules?;

        self.storage = Serialized{
            schedules, users, updated_at: Utc::now().timestamp(),
        };

        self.write_to_disk().await?;

        Ok(())
    }

    fn get_storage_file() -> PathBuf {
        let mut dir = Self::get_storage_dir(); 
        dir.push("storage.json");
        
        dir
    }

    fn get_storage_dir() -> PathBuf {
        let mut home = dirs::home_dir().expect("could not find home directory");
        home.push(".pd-quick-override");
        
        home
    }

    async fn write_to_disk(&self) -> Result<(), Box<dyn Error>> {
        let mut file = tokio::fs::File::create(Self::get_storage_file()).await?;

        let jstring = serde_json::to_string(&self.storage)?;

        file.write_all(jstring.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    pub async fn do_file_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut f = tokio::fs::File::open(Self::get_storage_file()).await?;

        let mut out = String::new();
        f.read_to_string(&mut out).await?;

        let parsed : Serialized = serde_json::from_str(out.as_str())?;

        self.storage = parsed;
        
        Ok(())
    }
}