//! PostgreSQL Database Library Benchmark Suite
//!
//! This crate provides a comprehensive and fair benchmark comparison between:
//! - tokio-postgres (low-level async driver)
//! - sqlx (compile-time checked SQL)
//! - sea-orm (async ORM)
//! - diesel (sync ORM with type safety)
//! - clorinde (code generation from SQL queries)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod bench_diesel;
pub mod bench_seaorm;
pub mod bench_sqlx;
pub mod bench_tokio_postgres;
pub mod bench_clorinde;

/// Database connection URL
pub const DATABASE_URL: &str = "postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db";

/// User model for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Post model for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: String,
    pub view_count: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Comment model for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Tag model for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// User with posts for join queries
#[derive(Debug, Clone)]
pub struct UserWithPosts {
    pub user: User,
    pub posts: Vec<Post>,
}

/// Post with comments for nested join queries
#[derive(Debug, Clone)]
pub struct PostWithComments {
    pub post: Post,
    pub comments: Vec<Comment>,
}

/// Input for creating a new user
#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: Option<i32>,
}

impl NewUser {
    pub fn generate(index: usize) -> Self {
        Self {
            username: format!("bench_user_{}", index),
            email: format!("bench_user_{}@benchmark.com", index),
            first_name: format!("First{}", index),
            last_name: format!("Last{}", index),
            age: Some((20 + (index % 60)) as i32),
        }
    }
}

/// Input for creating a new post
#[derive(Debug, Clone)]
pub struct NewPost {
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: String,
}

impl NewPost {
    pub fn generate(user_id: Uuid, index: usize) -> Self {
        Self {
            user_id,
            title: format!("Benchmark Post Title {}", index),
            content: format!("This is the content for benchmark post number {}. It contains enough text to simulate a realistic blog post with multiple paragraphs of content that would be typical in a real-world application.", index),
            status: if index % 3 == 0 { "draft" } else { "published" }.to_string(),
        }
    }
}

/// Input for creating a new comment
#[derive(Debug, Clone)]
pub struct NewComment {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}

impl NewComment {
    pub fn generate(post_id: Uuid, user_id: Uuid, index: usize) -> Self {
        Self {
            post_id,
            user_id,
            content: format!("This is benchmark comment number {} with some realistic content.", index),
        }
    }
}

/// Benchmark sizes for fair comparison
#[derive(Debug, Clone, Copy)]
pub struct BenchmarkSizes {
    pub small: usize,
    pub medium: usize,
    pub large: usize,
    pub xlarge: usize,
}

impl Default for BenchmarkSizes {
    fn default() -> Self {
        Self {
            small: 10,
            medium: 100,
            large: 1000,
            xlarge: 5000,
        }
    }
}

/// Heavy workload configuration
#[derive(Debug, Clone, Copy)]
pub struct HeavyWorkloadConfig {
    pub concurrent_connections: usize,
    pub operations_per_connection: usize,
    pub mixed_read_write_ratio: f64, // 0.0 = all writes, 1.0 = all reads
}

impl Default for HeavyWorkloadConfig {
    fn default() -> Self {
        Self {
            concurrent_connections: 50,
            operations_per_connection: 100,
            mixed_read_write_ratio: 0.8, // 80% reads, 20% writes
        }
    }
}

/// Trait for database benchmarks - ensures fair comparison
#[allow(async_fn_in_trait)]
pub trait DatabaseBenchmark {
    type Connection;
    type Error: std::fmt::Debug;

    /// Connect to the database
    async fn connect() -> Result<Self::Connection, Self::Error>;

    /// Insert a single user
    async fn insert_user(conn: &Self::Connection, user: &NewUser) -> Result<Uuid, Self::Error>;

    /// Insert multiple users in a batch
    async fn insert_users_batch(conn: &Self::Connection, users: &[NewUser]) -> Result<Vec<Uuid>, Self::Error>;

    /// Select a user by ID
    async fn select_user_by_id(conn: &Self::Connection, id: Uuid) -> Result<Option<User>, Self::Error>;

    /// Select users with limit
    async fn select_users_limit(conn: &Self::Connection, limit: i64) -> Result<Vec<User>, Self::Error>;

    /// Select users with complex filter
    async fn select_users_filtered(conn: &Self::Connection, min_age: i32, max_age: i32, limit: i64) -> Result<Vec<User>, Self::Error>;

    /// Update a user
    async fn update_user(conn: &Self::Connection, id: Uuid, first_name: &str, last_name: &str) -> Result<bool, Self::Error>;

    /// Delete a user
    async fn delete_user(conn: &Self::Connection, id: Uuid) -> Result<bool, Self::Error>;

    /// Insert a post
    async fn insert_post(conn: &Self::Connection, post: &NewPost) -> Result<Uuid, Self::Error>;

    /// Select posts with user join
    async fn select_posts_with_user(conn: &Self::Connection, limit: i64) -> Result<Vec<(Post, User)>, Self::Error>;

    /// Complex join: users -> posts -> comments
    async fn select_users_posts_comments(conn: &Self::Connection, limit: i64) -> Result<Vec<(User, Post, Comment)>, Self::Error>;

    /// Aggregate query: count posts per user
    async fn count_posts_per_user(conn: &Self::Connection) -> Result<Vec<(Uuid, i64)>, Self::Error>;

    /// Transaction: insert user and posts atomically
    async fn insert_user_with_posts(conn: &Self::Connection, user: &NewUser, posts: &[NewPost]) -> Result<Uuid, Self::Error>;

    /// Clean up benchmark data
    async fn cleanup(conn: &Self::Connection) -> Result<(), Self::Error>;
}
