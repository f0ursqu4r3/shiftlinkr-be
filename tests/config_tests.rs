use std::env;
use be::config::Config;

mod common;

#[test]
fn test_config_from_env_with_defaults() {
    common::setup_test_env();
    
    // Store original values
    let original_values = [
        ("DATABASE_URL", env::var("DATABASE_URL").ok()),
        ("JWT_SECRET", env::var("JWT_SECRET").ok()),
        ("JWT_EXPIRATION_DAYS", env::var("JWT_EXPIRATION_DAYS").ok()),
        ("HOST", env::var("HOST").ok()),
        ("PORT", env::var("PORT").ok()),
        ("ENVIRONMENT", env::var("ENVIRONMENT").ok()),
    ];

    // Clear environment variables
    for (key, _) in &original_values {
        unsafe { env::remove_var(key); }
    }

    let config = Config::from_env().unwrap();

    assert_eq!(config.database_url, "sqlite:./shiftlinkr.db");
    assert_eq!(config.jwt_secret, "your-super-secret-jwt-key-change-this-in-production-12345");
    assert_eq!(config.jwt_expiration_days, 30);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.environment, "development");

    // Restore original values
    for (key, value) in original_values {
        if let Some(val) = value {
            unsafe { env::set_var(key, val); }
        }
    }
}

#[test]
fn test_config_from_env_with_custom_values() {
    common::setup_test_env();
    
    // Store original values
    let original_values = [
        ("DATABASE_URL", env::var("DATABASE_URL").ok()),
        ("JWT_SECRET", env::var("JWT_SECRET").ok()),
        ("JWT_EXPIRATION_DAYS", env::var("JWT_EXPIRATION_DAYS").ok()),
        ("HOST", env::var("HOST").ok()),
        ("PORT", env::var("PORT").ok()),
        ("ENVIRONMENT", env::var("ENVIRONMENT").ok()),
    ];

    // Set custom values
    unsafe {
        env::set_var("DATABASE_URL", "sqlite:./test.db");
        env::set_var("JWT_SECRET", "test-secret");
        env::set_var("JWT_EXPIRATION_DAYS", "7");
        env::set_var("HOST", "0.0.0.0");
        env::set_var("PORT", "3000");
        env::set_var("ENVIRONMENT", "production");
    }

    let config = Config::from_env_only().unwrap();

    assert_eq!(config.database_url, "sqlite:./test.db");
    assert_eq!(config.jwt_secret, "test-secret");
    assert_eq!(config.jwt_expiration_days, 7);
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 3000);
    assert_eq!(config.environment, "production");

    // Restore original values
    unsafe {
        for (key, value) in original_values {
            if let Some(val) = value {
                env::set_var(key, val);
            } else {
                env::remove_var(key);
            }
        }
    }
}

#[test]
fn test_config_environment_detection() {
    let production_config = Config {
        database_url: "test".to_string(),
        jwt_secret: "test".to_string(),
        jwt_expiration_days: 1,
        host: "localhost".to_string(),
        port: 8080,
        environment: "production".to_string(),
    };

    let development_config = Config {
        database_url: "test".to_string(),
        jwt_secret: "test".to_string(),
        jwt_expiration_days: 1,
        host: "localhost".to_string(),
        port: 8080,
        environment: "development".to_string(),
    };

    assert!(production_config.is_production());
    assert!(!production_config.is_development());
    
    assert!(!development_config.is_production());
    assert!(development_config.is_development());
}

#[test]
fn test_server_address_formatting() {
    let config = Config {
        database_url: "test".to_string(),
        jwt_secret: "test".to_string(),
        jwt_expiration_days: 1,
        host: "192.168.1.1".to_string(),
        port: 9000,
        environment: "test".to_string(),
    };

    assert_eq!(config.server_address(), "192.168.1.1:9000");
}

#[test]
fn test_config_invalid_port() {
    // Store original
    let original_port = env::var("PORT").ok();
    
    unsafe { env::set_var("PORT", "invalid_port"); }
    
    let config = Config::from_env().unwrap();
    
    // Should fall back to default
    assert_eq!(config.port, 8080);
    
    // Restore
    unsafe {
        if let Some(val) = original_port {
            env::set_var("PORT", val);
        } else {
            env::remove_var("PORT");
        }
    }
}

#[test]
fn test_config_invalid_jwt_expiration() {
    // Store original
    let original_exp = env::var("JWT_EXPIRATION_DAYS").ok();
    
    unsafe { env::set_var("JWT_EXPIRATION_DAYS", "invalid_number"); }
    
    let config = Config::from_env().unwrap();
    
    // Should fall back to default
    assert_eq!(config.jwt_expiration_days, 30);
    
    // Restore
    unsafe {
        if let Some(val) = original_exp {
            env::set_var("JWT_EXPIRATION_DAYS", val);
        } else {
            env::remove_var("JWT_EXPIRATION_DAYS");
        }
    }
}
