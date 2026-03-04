//domain/service/id_generator.rs
// idGenerator
// 2025/7/8

use crate::domain::value_object::user_id::UserId;

/// ID生成器のトレイト
pub trait IdGeneratorInterface: Send + Sync {
    fn generate(&self) -> String;
    fn generate_user_id(&self) -> UserId {
        UserId::new(self.generate())
    }
}

/// UUID v4を使用したID生成器
pub struct UuidGenerator;

impl IdGeneratorInterface for UuidGenerator {
    fn generate(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
