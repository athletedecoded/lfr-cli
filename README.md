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
```

### New Users

To create a new user and provision them a new instance, run

```
$ cargo run new --user <username> --group <iam_group> --size <size> --mtype <machine_type>
```

where: 
* user: username
* group: IAM group policy
* size: one of "xl", "2xl", "4xl"
* mtype: one of "gpu", "std"

### Instance

To retrieve instance details of an existing instance

```
$ cargo run get --instance <instance_name>
```

