use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SECRET_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("AWS Access Key ID", Regex::new(r"(?i)AKIA[0-9A-Z]{16}").unwrap()),
        ("AWS Secret Access Key", Regex::new(r#"(?i)(aws|s3|ses|sns|sqs|dynamodb|rds|redshift|elasticache|glacier|cloudfront|route53|iam|sts|cloudwatch|cloudformation|elasticbeanstalk|opsworks|codecommit|codedeploy|codepipeline|ec2|vpc|elb|autoscaling|cloudtrail|config|directconnect|directoryservice|emr|kinesis|lambda|machinelearning|inspector|iot|cognito|mobileanalytics|devicefarm|workspaces|workdocs|workmail|waf|acm|ssm|shield|batch|stepfunctions|glue|athena|rekognition|lex|polly|lightsail|gamelift|dms|sms|budgets|costandusagereport|cur|discovery|applicationdiscovery|applicationdiscoveryservice|codestar|xray|pinpoint|clouddirectory|guardduty|mq|mediaconvert|medialive|mediapackage|mediastore|mediatailor|appsync|greengrass|sagemaker|serverlessrepo|servicecatalog|servicediscovery|transcribe|translate|cloud9|autoscalingplans|fms|secretsmanager|dlm|eks|macie|neptune|pi|ram|robomaker|signer|fsx|mediaconnect|securityhub|appmesh|licensemanager|kafka|apigatewaymanagementapi|apigatewayv2|docdb|backup|groundstation|managedblockchain|textract|iotevents|iotthingsgraph|iot1click|iot1clickdevices|iot1clickprojects|iotanalytics|mediaconnect|mediaconvert|medialive|mediapackage|mediastore|mediatailor|appsync|greengrass|sagemaker|serverlessrepo|servicecatalog|servicediscovery|transcribe|translate|cloud9|autoscalingplans|fms|secretsmanager|dlm|eks|macie|neptune|pi|ram|robomaker|signer|fsx|mediaconnect|securityhub|appmesh|licensemanager|kafka|apigatewaymanagementapi|apigatewayv2|docdb|backup|groundstation|managedblockchain|textract|iotevents|iotthingsgraph|iot1click|iot1clickdevices|iot1clickprojects|iotanalytics).{0,20}['\"][0-9a-zA-Z/+]{40}['\"]"#).unwrap()),
        ("Google API Key", Regex::new(r"(?i)AIza[0-9A-Za-z\\-_]{35}").unwrap()),
        ("Slack Token", Regex::new(r"(?i)xox[baprs]-([0-9a-zA-Z]{10,48})?").unwrap()),
        ("GitHub Personal Access Token", Regex::new(r"(?i)gh[pousr]_[a-zA-Z0-9]{36}").unwrap()),
        ("Private Key", Regex::new(r"(?i)-----BEGIN [A-Z ]+ PRIVATE KEY-----").unwrap()),
    ];
}

pub struct SecurityCheckResult {
    pub path: std::path::PathBuf,
    pub secrets: Vec<String>,
}

pub fn scan_content(path: &std::path::Path, content: &str) -> Result<Option<SecurityCheckResult>> {
    let mut found_secrets = Vec::new();

    for (name, regex) in SECRET_PATTERNS.iter() {
        if regex.is_match(content) {
            found_secrets.push(name.to_string());
        }
    }

    if found_secrets.is_empty() {
        Ok(None)
    } else {
        Ok(Some(SecurityCheckResult {
            path: path.to_path_buf(),
            secrets: found_secrets,
        }))
    }
}
