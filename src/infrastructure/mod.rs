//infrastructure/mod.rs
// インフラストラクチャレイヤー
// 2025/7/8

pub mod cache;
pub mod config;
pub mod cqrs;
pub mod database;
pub mod di;
// pub mod grpc;  // Temporarily disabled - requires protoc
pub mod repository;
pub mod web;
