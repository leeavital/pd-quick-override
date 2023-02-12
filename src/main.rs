use std::{
    collections::HashMap,
    io::{self, Read, Write},
    vec,
};

use chrono::TimeZone;
use clap::{Args, Parser, Subcommand, ValueEnum};
use client::Client;

mod client;
mod fuzzyselect;
mod timeparse;

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
    },
    ResetApiKey {
        
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { at } => {
            let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
            let now = chrono::Utc::now().timestamp();
            tz.timestamp_opt(now, 0).unwrap();
            let (to, from) = timeparse::parse(tz, tz.timestamp_opt(now, 0).unwrap(), at.as_str())
                .unwrap_or_else(|e| {
                    println!("cannot parse: {}", e);
                    std::process::exit(1);
                });
            println!("parsed {} to: {} {}", at, to, from);

            let client = Client::new().expect("could not open pagerduty client");

            let s = client.get_schedules().await.unwrap();
            println!("{:?}", s);

            let (users, mut schedules) = tokio::join!(client.get_users(), client.get_schedules(),);

            let mut users_by_email = HashMap::new();
            users.unwrap().into_iter().for_each(|u| {
                users_by_email.insert(u.email.clone(), u);
            });
            let selected_user = fuzzyselect::select(users_by_email).expect("could not read it");

            schedules = Ok(vec![client::Schedule {
                id: "testing".to_string(),
                name: "testing_name".to_string(),
            }]);
            let mut schedules_by_name = HashMap::new();
            schedules.unwrap().into_iter().for_each(|s| {
                schedules_by_name.insert(s.name.clone(), s);
            });
            let selected_schedule =
                fuzzyselect::select(schedules_by_name).expect("could not read it");

            println!("will create override on user {:?} for schedule {:?} from {} to {}, confirm to continue", selected_user, selected_schedule, from, to );
            if confirm() {
                client.create_schedule_override(selected_user, selected_schedule, from, to).await.expect("could not create override");
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
