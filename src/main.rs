use std::fs::File;
use std::io::Write;
use clap::{Parser, ArgAction};
use aws_sdk_iam::Client as IamClient;
use aws_sdk_lightsail::Client as LightsailClient;
use aws_sdk_secretsmanager::Client as SecretsClient;

use lfr_cli::{
    create_instance, delete_instance, get_instance,
    create_group, delete_group,
    create_user, delete_user, delete_user_instances,
    build_instance_config, build_iam_config
};

#[derive(Parser)]
//add extended help
#[clap(
version = "1.0",
author = "Kahlia Hogg",
about = "AWS LFR CLI",
after_help = "Example: cargo run new --user <username> --group <iam_group> --size <size> --mtype <machine_type>"
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
        group: String,
        #[clap(short, long)]
        size: String,
        #[clap(short, long)]
        mtype: String
    },
    Get {
        #[clap(short, long)]
        instance: Option<String>,
        #[clap(short, long, action = ArgAction::SetTrue)]
        key: Option<bool>
    },
    Instance {
        #[clap(short, long)]
        user: String,
        #[clap(short, long)]
        size: String,
        #[clap(short, long)]
        mtype: String
    },
    Delete {
        #[clap(short, long)]
        instance: Option<String>,
        #[clap(short, long)]
        user: Option<String>,
        #[clap(short, long)]
        group: Option<String>
    },
    Group {
        #[clap(short, long)]
        name: String
    }
}

#[tokio::main]
async fn main() {
    // Parse CLI args
    let args = Cli::parse();
    // Load AWS credentials from .env file
    dotenv::dotenv().ok();
    let config = aws_config::from_env().load().await;
    // Instantiate clients
    let lfr_client = LightsailClient::new(&config);
    let iam_client = IamClient::new(&config);
    let secrets_client = SecretsClient::new(&config);
    // Match on subcommand
    match args.command {
        Some(Commands::New { user, group, size, mtype }) => {
            // Create instance
            let zone = dotenv::var("LFR_ZONE").expect("LFR_ZONE not set");
            let instance_config = build_instance_config(&user, &size, &mtype, &zone);
            let instance_details = create_instance(lfr_client.clone(), instance_config).await;
            let arn = instance_details.instance.unwrap().arn.unwrap();
            // Create IAM User
            let iam_config = build_iam_config(&user, &group, &arn);
            let _ = create_user(iam_client.clone(), secrets_client.clone(), iam_config).await;
        },
        Some(Commands::Get { instance, key }) => {
            // Get instance details
            if let Some(instance_name) = instance {
                let instance_details = get_instance(lfr_client.clone(), &instance_name).await;
                println!("Instance details: {:?}", instance_details);
            } else if let Some(_key) = key {
                let keypair = lfr_client.download_default_key_pair().send().await.unwrap();
                if let Some(private_key) = keypair.private_key_base64 {
                    match File::create("dkp_rsa") {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(private_key.as_bytes()) {
                                println!("ERROR: Failed to write private key to file: {:?}", e);
                            } else {
                                println!("SUCCESS: Private key written to dkp_rsa");
                            }
                        }
                        Err(e) => println!("ERROR: Failed to create file: {:?}", e),
                    }
                } else {
                    println!("ERROR: No private key found in response.");
                }
            } else {
                // Handle the case where none of the options are supplied
                println!("ERROR: Incorrect arguments supplied.");
            }
        },
        Some(Commands::Instance { user, size, mtype }) => {
            // Create new instance
            let zone = dotenv::var("LFR_ZONE").expect("LFR_ZONE not set");
            let instance_config = build_instance_config(&user, &size, &mtype, &zone);
            let instance_details = create_instance(lfr_client.clone(), instance_config).await;
            let arn = instance_details.instance.unwrap().arn.unwrap();
            // ToDo: Auto add instance arn to policy
            println!("SUCCESS: Manually add instance arn {} to policy 'lfr-{}-access'", &arn, &user);
        },
        Some(Commands::Delete {instance, user, group}) => {
            // Delete single instance
            if let Some(instance_name) = instance {
                let _resp = delete_instance(lfr_client.clone(), &instance_name).await;
                println!("SUCCESS: Deleted instance: {}", &instance_name);
            } else if let Some(group_name) = group {
                if let Some(user_name) = user {
                    // Delete user instances and iam
                    delete_user_instances(lfr_client.clone(), &user_name).await;
                    let _ = delete_user(iam_client.clone(), &user_name, &group_name).await;
                    println!("SUCCESS: Deleted user: {}", user_name);
                } else {
                    // Delete entire group (all users + their instances)
                    let account_id = dotenv::var("AWS_ACCOUNT_ID").expect("AWS_ACCOUNT_ID not set");
                    let _ = delete_group(iam_client.clone(), lfr_client.clone(), &group_name, &account_id).await;
                    println!("SUCCESS: Deleted group: {}", group_name);
                }
            } else {
                // Handle the case where none of the options are supplied
                println!("ERROR: Incorrect arguments supplied.");
            }
        },
        Some(Commands::Group {name}) => {
            let account_id = dotenv::var("AWS_ACCOUNT_ID").expect("AWS_ACCOUNT_ID not set");
            let _ = create_group(iam_client.clone(), &name, &account_id).await;
            println!("SUCCESS: Created new group {}", &name);
        },
        None => {
            println!("No subcommand was used");
        }
    }
}