use log::info;
use rand::Rng;

use crate::config::Config;

const FIXED_OTP: &str = "12345"; // Constant fixed OTP for non-production environments

// Generate OTP (One-Time Password)
pub fn generate_otp(config: &Config) -> String {
    // Check if environment is production
    if config.env.eq_ignore_ascii_case("production") {
        info!("Production environment detected, generating OTP");
        let mut rng = rand::thread_rng();
        let range = rand::distributions::Uniform::new(100000, 999999);
        let otp = rng.sample(range); // Generate a random OTP within the specified range
        return otp.to_string(); // Convert OTP to string and return
    }

    info!("Local environment detected, using fixed OTP");
    FIXED_OTP.to_string() // Use fixed OTP for non-production environments and return
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{sms::Sms, smtp::Smtp, Config, Database, Jwt, Transactions};

    // Helper function to create a configuration with the specified environment
    fn create_config(env: &str) -> Config {
        Config {
            env: env.to_string(),
            host: String::new(),
            domain: String::new(),
            port: 0,
            database: Database {
                url: String::new(),
                connections: 0,
                transactions: Transactions {
                    url: String::new(),
                    connections: 0,
                },
            },
            jwt: Jwt {
                secret: String::new(),
            },
            cors: String::new(),
            sms: Sms {
                enable: false,
                account: String::new(),
                from: String::new(),
                token: String::new(),
            },
            smtp: Smtp {
                server: String::new(),
                username: String::new(),
                password: String::new(),
                port: "1025".to_string(),
                encryption: String::new(),
                from: String::new(),
            },
        }
    }

    #[test]
    fn test_generate_otp_non_production() {
        // Create a non-production configuration
        let config = create_config("local");

        // Generate OTP
        let otp = generate_otp(&config);

        // Ensure the OTP is the fixed OTP
        assert_eq!(otp, FIXED_OTP);
    }

    #[test]
    fn test_generate_otp_production() {
        // Create a production configuration
        let config = create_config("production");

        // Generate OTP
        let otp = generate_otp(&config);

        // Ensure the OTP is a 6-digit number
        let otp_num: u32 = otp.parse().expect("OTP should be a number");
        assert!(otp_num >= 100000 && otp_num <= 999999);
    }
}
