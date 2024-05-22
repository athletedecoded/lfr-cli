use tokio::time::sleep;
use std::time::Duration;
use aws_sdk_iam::Client as IamClient;
use aws_sdk_iam::operation::attach_group_policy::AttachGroupPolicyOutput;
use aws_sdk_iam::operation::get_user::GetUserOutput;
use aws_sdk_iam::operation::delete_user::DeleteUserOutput;
use aws_sdk_iam::operation::delete_group::DeleteGroupOutput;
use aws_sdk_lightsail::Client as LightsailClient;
use aws_sdk_lightsail::types::{StopInstanceOnIdleRequest, AddOnRequest, AddOnType};
use aws_sdk_lightsail::operation::get_instance::GetInstanceOutput;
use aws_sdk_lightsail::operation::get_instances::GetInstancesOutput;
use aws_sdk_lightsail::operation::delete_instance::DeleteInstanceOutput;
use aws_sdk_secretsmanager::Client as SecretsClient;

pub struct InstanceConfig {
    pub name: String,
    pub zone: String,
    pub blueprint_id: String,
    pub bundle_id: String,
    pub idle_threshold: String,
    pub idle_duration: String
}

pub struct IamConfig {
    pub user: String,
    pub group: String,
    pub arn:  String
}

pub fn build_instance_config(user: &str, size: &str, mtype: &str, zone: &str) -> InstanceConfig{
    let bundle_id = match mtype {
        "gpu" => format!("gpu_nvidia_{size}_1_0"),
        "std" => format!("app_standard_{size}_1_0"),
        _ => {
            println!("Invalid machine type, must be gpu or std");
            std::process::exit(1);
        }
    };
    // Return Instance Config
    InstanceConfig {
        name: format!("{user}-{mtype}-{size}"),
        zone: zone.to_string(),
        blueprint_id: "lfr_ubuntu_1_0".to_string(),
        bundle_id: bundle_id,
        idle_threshold: "1".to_string(),
        idle_duration: "20".to_string(),
    }
}

pub fn build_iam_config(user: &str, group: &str, arn: &str) -> IamConfig {
    IamConfig {
        user: user.to_string(),
        group: group.to_string(),
        arn: arn.to_string()
    }
}
pub async fn get_instance(lfr_client: LightsailClient, instance_name: &str) -> GetInstanceOutput {
    lfr_client.get_instance().instance_name(instance_name).send().await.unwrap()
}
pub async fn get_all_instances(lfr_client: LightsailClient) -> GetInstancesOutput {
    lfr_client.get_instances().send().await.unwrap()
}

pub async fn delete_instance(lfr_client: LightsailClient, instance_name: &str) -> DeleteInstanceOutput {
    lfr_client.delete_instance().instance_name(instance_name).force_delete_add_ons(true).send().await.unwrap()
}

pub async fn get_user(iam_client: IamClient, user: &str) -> GetUserOutput {
    iam_client.get_user().user_name(user).send().await.unwrap()
}
pub async fn delete_user(iam_client: IamClient, user: &str, group: &str) -> DeleteUserOutput {
    // Remove user from group
    let _ = iam_client.remove_user_from_group().user_name(user).group_name(group).send().await.unwrap();
    // Delete login profile
    let _ = iam_client.delete_login_profile().user_name(user).send().await.unwrap();
    // Delete user access policy
    let user_policy = format!("lfr-{}-access", user);
    let _ = iam_client.delete_user_policy().user_name(user).policy_name(user_policy).send().await.unwrap();
    // Delete IAM account
    iam_client.delete_user().user_name(user).send().await.unwrap()
}

pub async fn delete_user_instances(lfr_client: LightsailClient, user: &str) {
    let all_instances = get_all_instances(lfr_client.clone()).await;
    let user_instances: Vec<String> = all_instances.instances.clone()
        .unwrap_or_default()  // Provide a default if instances is None
        .into_iter()  // Move ownership if you want to consume instances
        .filter_map(|instance| {
            instance.name.as_ref().and_then(|name| {
                let mut parts = name.split('-');
                if let Some(username) = parts.next() {
                    if username == user {
                        return Some(name.clone());
                    }
                }
                None
            })
        })
        .collect();
    for instance in user_instances {
        let _ = delete_instance(lfr_client.clone(), &instance).await;
        println!("SUCCESS: Deleted instance {}", &instance);
    }
}

pub async fn create_group(iam_client: IamClient, group: &str, account_id: &str) -> AttachGroupPolicyOutput {
    // Create group
    let _ = iam_client.create_group().group_name(group).send().await.unwrap();
    // Attach student access policy
    let group_policy = format!("arn:aws:iam::{account_id}:policy/lfr-student-access");
    iam_client.attach_group_policy().group_name(group).policy_arn(group_policy).send().await.unwrap()
}

pub async fn delete_group(iam_client: IamClient, lfr_client: LightsailClient, group: &str, account_id: &str) -> DeleteGroupOutput {
    let all_users = iam_client.get_group().group_name(group).send().await.unwrap();
    for user in all_users.users {
        // Delete user instances
        let _ = delete_user_instances(lfr_client.clone(), &user.user_name).await;
        // Delete user iam
        let _ = delete_user(iam_client.clone(), &user.user_name, group).await;
        println!("SUCCESS: Deleted user {} from group {}", user.user_name, group);
    }
    // Detach group policy
    let group_policy = format!("arn:aws:iam::{account_id}:policy/lfr-student-access");
    let _ = iam_client.detach_group_policy().group_name(group).policy_arn(&group_policy).send().await.unwrap();
    // Delete group
    iam_client.delete_group().group_name(group).send().await.unwrap()
}

pub fn build_policy_doc(instance_arn: String) -> String {
    format!(r#"{{
        "Version": "2012-10-17",
        "Statement": [
            {{
                "Effect": "Allow",
                "Action": [
                    "lightsail:*"
                ],
                "Resource": "{instance_arn}"
            }}
        ]
    }}"#)
}
pub async fn probe_state(lfr_client: LightsailClient, instance_name: &str, state: &str) -> bool {
    let mut in_state = false;
    while !in_state {
        let response = lfr_client.get_instance_state().instance_name(instance_name).send().await.unwrap();
        let instance_state = response.state.unwrap().name.unwrap();
        println!("Instance state: {}", instance_state);
        if instance_state == state {
            in_state = true;
        }
        sleep(Duration::from_secs(5)).await;
    }
    in_state
}

pub async fn create_instance(lfr_client: LightsailClient, instance_config: InstanceConfig) -> GetInstanceOutput {
    // Stop Instance on Idle
    let idle_request = StopInstanceOnIdleRequest::builder()
        .threshold(instance_config.idle_threshold)
        .duration(instance_config.idle_duration)
        .build();
    let add_on_request = AddOnRequest::builder()
        .add_on_type(AddOnType::StopInstanceOnIdle)
        .stop_instance_on_idle_request(idle_request)
        .build()
        .unwrap();
    // Create instance
    let instance_created = match lfr_client.create_instances()
        .instance_names(&instance_config.name)
        .availability_zone(instance_config.zone)
        .blueprint_id(instance_config.blueprint_id)
        .bundle_id(instance_config.bundle_id)
        .add_ons(add_on_request)
        .send()
        .await {
        Ok(_) => {
            println!("SUCCESS: Created instance {}", &instance_config.name);
            true
        },
        Err(error) => {
            println!("ERROR: Failed to create instance {}", &instance_config.name);
            println!("{:?}", error);
            false
        }
    };
    // If instance created, check running --> stop
    if instance_created {
        // Probe instance state at 5 sec intervals
        let is_running = probe_state(lfr_client.clone(), &instance_config.name, "running").await;
        // Running --> Stop
        if is_running {
            let _ = lfr_client.stop_instance().instance_name(&instance_config.name).send().await.unwrap();
            let is_stopping = probe_state(lfr_client.clone(), &instance_config.name, "stopping").await;
            if is_stopping {
                println!("SUCCESS: Stopping instance {}", &instance_config.name);
            } else {
                println!("ERROR: Unable to stop instance {}", &instance_config.name);
            }
        } else {
            println!("ERROR: Instance {} is not running.", &instance_config.name);
        }
    } else {
        println!("ERROR: Unable to create instance {}", &instance_config.name);
        std::process::exit(1);
    }
    // Return instance details
    get_instance(lfr_client.clone(), &instance_config.name).await
}

pub async fn create_user(iam_client: IamClient, secrets_client: SecretsClient, iam_config: IamConfig) -> GetUserOutput {
    let user_created = match iam_client.create_user()
        .user_name(&iam_config.user)
        .send()
        .await {
            Ok(_) => {
            println!("SUCCESS: Created user {}", &iam_config.user);
            true
        },
        Err(error) => {
        println!("ERROR: Failed to create user {}", &iam_config.user);
        println!("{:?}", error);
        false
        }
    };
    // If user created
    if user_created {
        // Add to group
        let _ = iam_client.add_user_to_group()
            .user_name(&iam_config.user)
            .group_name(&iam_config.group)
            .send()
            .await
            .unwrap();
        println!("SUCCESS: User added to group {}", &iam_config.group);
        // Create login profile
        let password = secrets_client.get_random_password()
            .password_length(8)
            .send()
            .await
            .unwrap()
            .random_password
            .unwrap();
        let _ = iam_client.create_login_profile()
            .user_name(&iam_config.user)
            .password(&password)
            .password_reset_required(true)
            .send()
            .await
            .unwrap();
        println!("SUCCESS: Created login profile");
        // Add user access policy
        let user_policy = format!("lfr-{}-access", &iam_config.user);
        let policy_document = build_policy_doc(iam_config.arn);
        let _ = iam_client.put_user_policy()
            .user_name(&iam_config.user)
            .policy_name(&user_policy)
            .policy_document(policy_document)
            .send()
            .await
            .unwrap();
        println!("SUCCESS: Added user access policy {}", &user_policy);
        println!("SUCCESS: Created user '{}' with unique onetime password: {}", &iam_config.user, password);
    } else {
        println!("ERROR: Unable to create user {}", &iam_config.user);
        std::process::exit(1);
    }
    // Return user details
    get_user(iam_client.clone(), &iam_config.user).await
}



