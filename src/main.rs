use std::error::Error;
use clap::Parser;
use std::time::Duration;
use tokio::time::sleep;
use aws_sdk_lightsail::Client as LightsailClient;
use aws_sdk_lightsail::error::ProvideErrorMetadata;
use aws_sdk_lightsail::operation::get_instance::GetInstanceOutput;
use aws_sdk_lightsail::types::{StopInstanceOnIdleRequest, AddOnRequest, AddOnType};

use lfr_cli::{InstanceConfig, create_instance, get_instance, build_config};

#[derive(Parser)]
//add extended help
#[clap(
version = "1.0",
author = "Kahlia Hogg",
about = "AWS LFR CLI",
after_help = "Example: cargo run new --user <username> --size <size> --mtype <machine_type>"
)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    New {
        #[clap(short, long)]
        user: String,
        #[clap(short, long)]
        size: String,
        #[clap(short, long)]
        mtype: String
    },
    Get {
        #[clap(short, long)]
        instance: String
    }
}

#[tokio::main]
async fn main() {
    // Parse CLI args
    let args = Cli::parse();
    // Load AWS credentials from .env file
    dotenv::dotenv().ok();
    let config = aws_config::from_env().load().await;
    let lfr_client = LightsailClient::new(&config);
    // Match on subcommand
    match args.command {
        Some(Commands::New { user, size, mtype }) => {
            // Create instance
            let zone = "us-east-2a";
            let instance_config = build_config(&user, &size, &mtype, &zone);
            let instance_details = create_instance(lfr_client.clone(), instance_config).await;
            let arn = instance_details.instance.unwrap().arn.unwrap();
            // Create IAM Role
            println!("Instance ARN: {arn}");
        },
        Some(Commands::Get { instance }) => {
            // Get instance detais
            let instance_details = get_instance(lfr_client.clone(), &instance).await;
            println!("Instance details: {:?}", instance_details);
        }
        None => {
            println!("No subcommand was used");
        }
    }
}