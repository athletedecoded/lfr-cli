## Lightsail For Research CLI Tool

[![CI/CD](https://github.com/athletedecoded/lfr-cli/actions/workflows/cicd.yml/badge.svg)](https://github.com/athletedecoded/lfr-cli/actions/workflows/cicd.yml)


![image](banner.png)

### Setup

**Create Policy: `lfr-cli-admin`**

IAM Console > Policies > Create Policy > JSON Editor:

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

**Create User: `lfr-cli`**

IAM Console > Users > Create User > Attach policies directly: `lfr-cli-admin`

Create access key for 

**Configure Environment**

In `sample.env` add lfr-cli access key and AWS account ID values

```
# sample.env
AWS_ACCESS_KEY_ID=<LFR_CLI_ACCESS_KEY>
AWS_SECRET_ACCESS_KEY=<LFR_CLI_SECRET_KEY>
AWS_ACCOUNT_ID=<DUKE_ACCOUNT_ID>
AWS_DEFAULT_REGION=us-east-2
LFR_ZONE=us-east-2a
```

Move/rename `sample.env` to `.env`

```
$ mv sample.env .env
```

---

### Build Binary

```
$ cargo build --release
```

The binary is now available in `/target/release`

A prebuilt binary is also available at: [https://github.com/athletedecoded/lfr-cli/releases](https://github.com/athletedecoded/lfr-cli/releases)

---

### Useage

**Assumptions/Rules:**

* A user may only be attached to one group


To run in "developer mode" use `cargo run` instead of `./lfr`

#### CREATE

where:
* user: Duke NetID
* group: IAM user group name
* size: one of "xl", "2xl", "4xl"
* mtype: one of "gpu", "std"

**Create New User + Provision New Instance**

```
$ ./lfr new --user <username> --group <iam_group> --size <machine_size> --mtype <machine_type>
```

**Create New Instance for Existing User**

NB: Currently this requires manually adding the new arn to the existing user access policy

```
$ ./lfr instance --user <username> --size <machine_size> --mtype <machine_type>
```

**Create New Group**

NB: Will create new IAM group and attach `lfr-student-access` policy

```
$ ./lfr group --name <group_name>
```

--- 

#### GET

**Get Instance Details**

```
$ ./lfr get --instance <instance_name>
```

**Get SSH Key**

```
$ ./lfr get --key
```

--- 

#### DELETE

**Delete Instance**

NB: Instance must exist and be in stopped state

```
$ ./lfr delete --instance <instance_name>
```

**Delete User**

NB: Will delete user account and associated instances. Must supply group name.

```
$ ./lfr delete --user <user_name> --group <group_name>
```

**Delete Group**

```
$ ./lfr delete --group <group_name>
```

---

### Future Features

* [ ] Automatically add multiple instance arns to existing user access policy