//shared/utils/password_hasher.rs
// パスワードハッシュ化
// 2025/7/8

use crate::shared::error::infrastructure_error::InfrastructureError;

/// Password hashing interface
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, InfrastructureError>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool, InfrastructureError>;
}

/// Simple password hasher implementation (for development)
/// In production, use bcrypt or argon2
pub struct SimplePasswordHasher;

impl SimplePasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimplePasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher for SimplePasswordHasher {
    fn hash(&self, password: &str) -> Result<String, InfrastructureError> {
        // Simple hash for development - DO NOT USE IN PRODUCTION
        Ok(format!("hashed_{}", password))
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, InfrastructureError> {
        Ok(hash == format!("hashed_{}", password))
    }
}
