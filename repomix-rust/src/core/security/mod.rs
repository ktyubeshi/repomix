use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SECRET_PATTERNS: Vec<(&'static str, Regex)> = vec![
        // AWS
        ("AWS Access Key ID", Regex::new(r"(?i)\b(A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}\b").unwrap()),
        ("AWS Secret Access Key", Regex::new(r#"(?i)(aws|s3|ses|sns|sqs|dynamodb|rds|redshift|elasticache|glacier|cloudfront|route53|iam|sts|cloudwatch|cloudformation|elasticbeanstalk|opsworks|codecommit|codedeploy|codepipeline|ec2|vpc|elb|autoscaling|cloudtrail|config|directconnect|directoryservice|emr|kinesis|lambda|machinelearning|inspector|iot|cognito|mobileanalytics|devicefarm|workspaces|workdocs|workmail|waf|acm|ssm|shield|batch|stepfunctions|glue|athena|rekognition|lex|polly|lightsail|gamelift|dms|sms|budgets|costandusagereport|cur|discovery|applicationdiscovery|applicationdiscoveryservice|codestar|xray|pinpoint|clouddirectory|guardduty|mq|mediaconvert|medialive|mediapackage|mediastore|mediatailor|appsync|greengrass|sagemaker|serverlessrepo|servicecatalog|servicediscovery|transcribe|translate|cloud9|autoscalingplans|fms|secretsmanager|dlm|eks|macie|neptune|pi|ram|robomaker|signer|fsx|mediaconnect|securityhub|appmesh|licensemanager|kafka|apigatewaymanagementapi|apigatewayv2|docdb|backup|groundstation|managedblockchain|textract|iotevents|iotthingsgraph|iot1click|iot1clickdevices|iot1clickprojects|iotanalytics).{0,20}['"][0-9a-zA-Z/+]{40}['"]"#).unwrap()),

        // Google
        ("Google API Key", Regex::new(r"(?i)AIza[0-9A-Za-z_\-]{35}").unwrap()),
        ("Google OAuth Access Token", Regex::new(r"(?i)ya29\.[0-9A-Za-z_\-]+").unwrap()),

        // GitHub
        ("GitHub Personal Access Token", Regex::new(r"(?i)\bgh[pousr]_[a-zA-Z0-9]{36}\b").unwrap()),
        ("GitHub Fine-Grained Personal Access Token", Regex::new(r"(?i)\bgithub_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}\b").unwrap()),
        ("GitHub OAuth Access Token", Regex::new(r"(?i)\bgho_[a-zA-Z0-9]{36}\b").unwrap()),

        // Slack
        ("Slack Token", Regex::new(r"(?i)xox[baprs]-([0-9a-zA-Z]{10,48})?").unwrap()),
        ("Slack Webhook", Regex::new(r"(?i)https://hooks\.slack\.com/services/T[a-zA-Z0-9_]{8}/B[a-zA-Z0-9_]{8}/[a-zA-Z0-9_]{24}").unwrap()),

        // Azure
        ("Azure Subscription ID", Regex::new(r"(?i)[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap()),

        // Heroku
        ("Heroku API Key", Regex::new(r"(?i)[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap()),

        // Stripe
        ("Stripe API Key", Regex::new(r"(?i)\b(?:sk|pk)_(?:test|live)_[a-zA-Z0-9]{24,99}\b").unwrap()),

        // Twilio
        ("Twilio API Key", Regex::new(r"(?i)\bSK[0-9a-fA-F]{32}\b").unwrap()),

        // Private Keys
        ("Private Key", Regex::new(r"(?i)-----BEGIN [A-Z ]+ PRIVATE KEY-----").unwrap()),
        ("PGP Private Key", Regex::new(r"(?i)-----BEGIN PGP PRIVATE KEY BLOCK-----").unwrap()),

        // Generic
        ("Generic API Key", Regex::new(r#"(?i)(api_key|apikey|secret|token)[\s':=]+(['"][a-zA-Z0-9_\-]{16,64}['"])"#).unwrap()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_aws_access_key() {
        let content = "My AWS key is AKIAIOSFODNN7EXAMPLE";
        let result = scan_content(Path::new("test.txt"), content).unwrap();
        assert!(result.is_some());
        let secrets = result.unwrap().secrets;
        assert!(secrets.contains(&"AWS Access Key ID".to_string()));
    }

    #[test]
    fn test_google_api_key() {
        let content = "AIzaSyD-example-key-1234567890123456789";
        let result = scan_content(Path::new("test.txt"), content).unwrap();
        assert!(result.is_some());
        let secrets = result.unwrap().secrets;
        assert!(secrets.contains(&"Google API Key".to_string()));
    }

    #[test]
    fn test_github_token() {
        let content = "ghp_123456789012345678901234567890123456";
        let result = scan_content(Path::new("test.txt"), content).unwrap();
        assert!(result.is_some());
        let secrets = result.unwrap().secrets;
        assert!(secrets.contains(&"GitHub Personal Access Token".to_string()));
    }

    #[test]
    fn test_private_key() {
        let content = "-----BEGIN RSA PRIVATE KEY-----";
        let result = scan_content(Path::new("test.txt"), content).unwrap();
        assert!(result.is_some());
        let secrets = result.unwrap().secrets;
        assert!(secrets.contains(&"Private Key".to_string()));
    }

    #[test]
    fn test_generic_api_key() {
        let content = "const api_key = '1234567890abcdef1234567890abcdef'";
        let result = scan_content(Path::new("test.txt"), content).unwrap();
        assert!(result.is_some());
        let secrets = result.unwrap().secrets;
        assert!(secrets.contains(&"Generic API Key".to_string()));
    }
}
