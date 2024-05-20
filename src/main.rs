use clap::Parser;
use aws_sdk_iam::Client as IamClient;
use aws_sdk_lightsail::Client as LightsailClient;
use aws_sdk_secretsmanager::Client as SecretsClient;

use lfr_cli::{create_instance, delete_instance, get_instance, get_all_instances, create_user, delete_user, build_instance_config, build_iam_config, build_policy_doc};

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
        instance: String
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
            let user_details = create_user(iam_client.clone(), secrets_client.clone(), iam_config).await;
        },
        Some(Commands::Get { instance }) => {
            // Get instance detais
            let instance_details = get_instance(lfr_client.clone(), &instance).await;
            println!("Instance details: {:?}", instance_details);
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
                // Delete user and their instances
                if let Some(user_name) = user {
                    let _ = delete_user(iam_client.clone(), &user_name, &group_name).await;
                    println!("SUCCESS: Deleted user: {}", user_name);
                    let all_instances = get_all_instances(lfr_client.clone()).await;
                    let user_instances: Vec<String> = all_instances.instances
                        .unwrap_or_default()  // Provide a default if instances is None
                        .into_iter()  // Move ownership if you want to consume instances
                        .filter_map(|instance| {
                            instance.name.as_ref().filter(|name| name.starts_with(&user_name)).cloned()  // Safely filter and clone the name
                        })
                        .collect();
                    for instance in user_instances {
                        let _ = delete_instance(lfr_client.clone(), &instance).await;
                        println!("SUCCESS: Deleted instance {}", &instance);
                    }
                } else {
                    // Delete entire group (all users + their instances)
                    println!("SUCCESS: Deleted group: {}", group_name);
                }
            } else {
                // Handle the case where none of the options are supplied
                println!("ERROR: Incorrect arguments supplied.");
            }
        },
        None => {
            println!("No subcommand was used");
        }
    }
}