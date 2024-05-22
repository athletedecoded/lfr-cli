## Lightsail For Research CLI Tool

### Setup

**IAM Role**

Create an admin user 'lfr-cli' and attach policy 'lfr-cli-admin' below: 

```
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "iam:*",
                "lightsail:*",
                "secretsmanager:GetRandomPassword"
            ],
            "Resource": "*"
        }
    ]
}
```

**Env**

1. Create access key for user 'lfr-cli'
2. Configure .env file

```
AWS_ACCESS_KEY_ID=<YOUR_ACCESS_KEY>
AWS_SECRET_ACCESS_KEY=<YOUR_SECRET_KEY>
AWS_DEFAULT_REGION=<YOUR_AWS_REGION>
LFR_ZONE=<LFR_AVAILABILITY_ZONE>
AWS_ACCOUNT_ID=<YOUR_ACCOUNT_ID>
```

### CLI Tool

**Create New User + Provision New Instance**

```
$ cargo run new --user <username> --group <iam_group> --size <machine_size> --mtype <machine_type>
```

where: 
* user: username
* group: IAM group policy
* size: one of "xl", "2xl", "4xl"
* mtype: one of "gpu", "std"


**Get Instance Details**

```
$ cargo run get --instance <instance_name>
```

**Create New Instance for Existing User**

NB: Currently this requires manually adding the new arn to the existing user access policy

```
$ cargo run instance --user <username> --size <machine_size> --mtype <machine_type>
```

**Create New Group**

NB: Will create with `lfr-student-access` policy
```
$ cargo run group --name <group_name>
```

**Delete Instance**

NB: Instance must exist and be in stopped state

```
$ cargo run delete --instance <instance_name>
```

**Delete User**

NB: Will delete user account and associated instances. Must supply group name.

```
$ cargo run delete --user <user_name> --group <group_name>
```

**Delete Group**

NB: 

### ToDos

*[ ] Debug profile failure "password doesn't contain symbol"
*[ ] Add create group if DNE
*[ ] Auto add arn to existing policy
