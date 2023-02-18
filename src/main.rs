use std::{
    collections::HashMap,
    io::{self, Write},
};


use chrono::TimeZone;
use clap::{Parser, Subcommand};
use client::Client;

mod client;
mod fuzzyselect;
mod timeparse;
mod persistence;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    Create {
        #[arg(short, long)]
        at: String,
        
        #[arg(short, long)]
        me: bool,

        #[arg(short, long)]
        time_zone: Option<String>,
    },
    ResetApiKey {
        
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { at , time_zone , me } => {
            let tz_string = time_zone.unwrap_or_else(|| {
                iana_time_zone::get_timezone().expect("could not find timezone")
            });

            let tz: chrono_tz::Tz = tz_string.parse().unwrap();
            
            let now = chrono::Utc::now().timestamp();
            tz.timestamp_opt(now, 0).unwrap();
            let (from, to) = timeparse::parse(&tz.timestamp_opt(now, 0).unwrap(), at.as_str())
                .unwrap_or_else(|e| {
                    println!("cannot parse: {}", e);
                    std::process::exit(1);
                });

            let client = Client::new().expect("could not open pagerduty client");
            let db = persistence::Database::load(&client).await.expect("could not load database");

            let mut users_by_email = HashMap::new();
            db.storage.users.iter().for_each(|u| {
                users_by_email.insert(u.email.clone(), u);
            });

            let current_user;
            let selected_user = if me {
                current_user =  client.get_me().await.unwrap();
                &current_user
            } else {
                fuzzyselect::select(users_by_email).expect("could not read it")
            };

            let mut schedules_by_name = HashMap::new();
            db.storage.schedules.iter().for_each(|s| {
                schedules_by_name.insert(s.name.clone(), s);
            });
            let selected_schedule =
                fuzzyselect::select(schedules_by_name).expect("could not read it");

            println!("will create override on user {selected_user} for schedule {selected_schedule} from {from} to {to}, confirm to continue");
            if confirm() {
                client.create_schedule_override(selected_user, selected_schedule, from, to).await.expect("could not create override");
                println!("Override created! Good luck! ")
            }
        },
        Commands::ResetApiKey {} => { 
            todo!();
        },
    }
}

fn confirm() -> bool {
    let sin = io::stdin();
    let mut answer = String::new();
    loop {
        print!("y(es)/n(o)? ");
        io::stdout().flush().expect("could not flush stdout");

        answer.clear();
        sin.read_line(&mut answer).expect("could not read stdin");

        let trimmed = answer.trim();
        if trimmed == "y" || trimmed == "yes" {
            return true;
        }

        if trimmed == "n" || trimmed == "no" {
            return false;
        }

        println!("got {}", trimmed);
    }
}
