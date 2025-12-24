//! tokio-postgres benchmark implementation

use crate::{Comment, NewComment, NewPost, NewUser, Post, User, DATABASE_URL};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

// Re-export deadpool types for pooled benchmarks
pub use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};

pub struct TokioPostgresBench;

impl TokioPostgresBench {
    pub async fn connect() -> Result<Client, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        
        Ok(client)
    }
    
    /// Create a deadpool connection pool for concurrent benchmarks
    pub fn create_pool(pool_size: usize) -> Pool {
        let mut cfg = Config::new();
        cfg.url = Some(DATABASE_URL.to_string());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        cfg.pool = Some(deadpool_postgres::PoolConfig {
            max_size: pool_size,
            ..Default::default()
        });
        
        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("Failed to create pool")
    }
    
    /// Get a client from the pool
    pub async fn get_pooled_client(pool: &Pool) -> Result<deadpool_postgres::Client, deadpool_postgres::PoolError> {
        pool.get().await
    }
    
    pub async fn insert_user(client: &Client, user: &NewUser) -> Result<Uuid, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO users (username, email, first_name, last_name, age) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id",
                &[&user.username, &user.email, &user.first_name, &user.last_name, &user.age],
            )
            .await?;
        Ok(row.get("id"))
    }
    
    pub async fn insert_users_batch(client: &Client, users: &[NewUser]) -> Result<Vec<Uuid>, tokio_postgres::Error> {
        let mut ids = Vec::with_capacity(users.len());
        
        // Use individual inserts for fair comparison
        // In a real scenario, you'd use COPY or batch statements
        for user in users {
            let row = client
                .query_one(
                    "INSERT INTO users (username, email, first_name, last_name, age) 
                     VALUES ($1, $2, $3, $4, $5) 
                     RETURNING id",
                    &[&user.username, &user.email, &user.first_name, &user.last_name, &user.age],
                )
                .await?;
            ids.push(row.get("id"));
        }
        
        Ok(ids)
    }
    
    pub async fn select_user_by_id(client: &Client, id: Uuid) -> Result<Option<User>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users WHERE id = $1",
                &[&id],
            )
            .await?;
        
        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            first_name: r.get("first_name"),
            last_name: r.get("last_name"),
            age: r.get("age"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }
    
    pub async fn select_users_limit(client: &Client, limit: i64) -> Result<Vec<User>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users ORDER BY created_at DESC LIMIT $1",
                &[&limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                username: r.get("username"),
                email: r.get("email"),
                first_name: r.get("first_name"),
                last_name: r.get("last_name"),
                age: r.get("age"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
    
    pub async fn select_users_filtered(
        client: &Client,
        min_age: i32,
        max_age: i32,
        limit: i64,
    ) -> Result<Vec<User>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users 
                 WHERE age >= $1 AND age <= $2 
                 ORDER BY age, username 
                 LIMIT $3",
                &[&min_age, &max_age, &limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                username: r.get("username"),
                email: r.get("email"),
                first_name: r.get("first_name"),
                last_name: r.get("last_name"),
                age: r.get("age"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
    
    pub async fn update_user(
        client: &Client,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<bool, tokio_postgres::Error> {
        let rows_affected = client
            .execute(
                "UPDATE users SET first_name = $1, last_name = $2, updated_at = NOW() WHERE id = $3",
                &[&first_name, &last_name, &id],
            )
            .await?;
        Ok(rows_affected > 0)
    }
    
    pub async fn delete_user(client: &Client, id: Uuid) -> Result<bool, tokio_postgres::Error> {
        let rows_affected = client
            .execute("DELETE FROM users WHERE id = $1", &[&id])
            .await?;
        Ok(rows_affected > 0)
    }
    
    pub async fn insert_post(client: &Client, post: &NewPost) -> Result<Uuid, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO posts (user_id, title, content, status) 
                 VALUES ($1, $2, $3, $4) 
                 RETURNING id",
                &[&post.user_id, &post.title, &post.content, &post.status],
            )
            .await?;
        Ok(row.get("id"))
    }
    
    pub async fn select_posts_with_user(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<(Post, User)>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT 
                    p.id as post_id, p.user_id, p.title, p.content, p.status, p.view_count,
                    p.created_at as post_created_at, p.updated_at as post_updated_at,
                    u.id as user_id, u.username, u.email, u.first_name, u.last_name, u.age,
                    u.created_at as user_created_at, u.updated_at as user_updated_at
                 FROM posts p
                 JOIN users u ON p.user_id = u.id
                 ORDER BY p.created_at DESC
                 LIMIT $1",
                &[&limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| {
                let post = Post {
                    id: r.get("post_id"),
                    user_id: r.get("user_id"),
                    title: r.get("title"),
                    content: r.get("content"),
                    status: r.get("status"),
                    view_count: r.get("view_count"),
                    created_at: r.get("post_created_at"),
                    updated_at: r.get("post_updated_at"),
                };
                let user = User {
                    id: r.get("user_id"),
                    username: r.get("username"),
                    email: r.get("email"),
                    first_name: r.get("first_name"),
                    last_name: r.get("last_name"),
                    age: r.get("age"),
                    created_at: r.get("user_created_at"),
                    updated_at: r.get("user_updated_at"),
                };
                (post, user)
            })
            .collect())
    }
    
    pub async fn select_users_posts_comments(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<(User, Post, Comment)>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT 
                    u.id as user_id, u.username, u.email, u.first_name, u.last_name, u.age,
                    u.created_at as user_created_at, u.updated_at as user_updated_at,
                    p.id as post_id, p.title, p.content, p.status, p.view_count,
                    p.created_at as post_created_at, p.updated_at as post_updated_at,
                    c.id as comment_id, c.content as comment_content, c.created_at as comment_created_at
                 FROM users u
                 JOIN posts p ON u.id = p.user_id
                 JOIN comments c ON p.id = c.post_id
                 ORDER BY u.created_at DESC, p.created_at DESC, c.created_at DESC
                 LIMIT $1",
                &[&limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| {
                let user = User {
                    id: r.get("user_id"),
                    username: r.get("username"),
                    email: r.get("email"),
                    first_name: r.get("first_name"),
                    last_name: r.get("last_name"),
                    age: r.get("age"),
                    created_at: r.get("user_created_at"),
                    updated_at: r.get("user_updated_at"),
                };
                let post = Post {
                    id: r.get("post_id"),
                    user_id: r.get("user_id"),
                    title: r.get("title"),
                    content: r.get("content"),
                    status: r.get("status"),
                    view_count: r.get("view_count"),
                    created_at: r.get("post_created_at"),
                    updated_at: r.get("post_updated_at"),
                };
                let comment = Comment {
                    id: r.get("comment_id"),
                    post_id: r.get("post_id"),
                    user_id: r.get("user_id"),
                    content: r.get("comment_content"),
                    created_at: r.get("comment_created_at"),
                };
                (user, post, comment)
            })
            .collect())
    }
    
    pub async fn count_posts_per_user(client: &Client) -> Result<Vec<(Uuid, i64)>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT u.id, COUNT(p.id) as post_count
                 FROM users u
                 LEFT JOIN posts p ON u.id = p.user_id
                 GROUP BY u.id
                 ORDER BY post_count DESC",
                &[],
            )
            .await?;
        
        Ok(rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }
    
    pub async fn insert_user_with_posts(
        client: &Client,
        user: &NewUser,
        posts: &[NewPost],
    ) -> Result<Uuid, tokio_postgres::Error> {
        // Note: tokio-postgres requires a mutable client for transactions
        // For benchmarking purposes, we'll do sequential inserts
        let user_id = Self::insert_user(client, user).await?;
        
        for post in posts {
            let mut post = post.clone();
            post.user_id = user_id;
            Self::insert_post(client, &post).await?;
        }
        
        Ok(user_id)
    }
    
    pub async fn cleanup(client: &Client) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "DELETE FROM users WHERE username LIKE 'bench_user_%'",
                &[],
            )
            .await?;
        Ok(())
    }
    
    // Additional methods for heavy workload benchmarks
    
    pub async fn insert_comment(client: &Client, comment: &NewComment) -> Result<Uuid, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO comments (post_id, user_id, content) 
                 VALUES ($1, $2, $3) 
                 RETURNING id",
                &[&comment.post_id, &comment.user_id, &comment.content],
            )
            .await?;
        Ok(row.get("id"))
    }
    
    pub async fn select_posts_by_status(
        client: &Client,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Post>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, user_id, title, content, status, view_count, created_at, updated_at 
                 FROM posts 
                 WHERE status = $1 
                 ORDER BY created_at DESC 
                 LIMIT $2",
                &[&status, &limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| Post {
                id: r.get("id"),
                user_id: r.get("user_id"),
                title: r.get("title"),
                content: r.get("content"),
                status: r.get("status"),
                view_count: r.get("view_count"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
    
    pub async fn increment_view_count(client: &Client, post_id: Uuid) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "UPDATE posts SET view_count = view_count + 1 WHERE id = $1",
                &[&post_id],
            )
            .await?;
        Ok(())
    }
    
    pub async fn search_users_by_name(
        client: &Client,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<User>, tokio_postgres::Error> {
        let pattern = format!("%{}%", pattern);
        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users 
                 WHERE first_name ILIKE $1 OR last_name ILIKE $1 
                 ORDER BY username 
                 LIMIT $2",
                &[&pattern, &limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                username: r.get("username"),
                email: r.get("email"),
                first_name: r.get("first_name"),
                last_name: r.get("last_name"),
                age: r.get("age"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
}

// ============================================================================
// Pooled versions for concurrent benchmarks
// ============================================================================

impl TokioPostgresBench {
    pub async fn pooled_insert_user(
        pool: &Pool,
        user: &NewUser,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let client = pool.get().await?;
        let row = client
            .query_one(
                "INSERT INTO users (username, email, first_name, last_name, age) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id",
                &[&user.username, &user.email, &user.first_name, &user.last_name, &user.age],
            )
            .await?;
        Ok(row.get("id"))
    }

    pub async fn pooled_select_user_by_id(
        pool: &Pool,
        id: Uuid,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let client = pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users WHERE id = $1",
                &[&id],
            )
            .await?;
        
        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            first_name: r.get("first_name"),
            last_name: r.get("last_name"),
            age: r.get("age"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn pooled_select_users_limit(
        pool: &Pool,
        limit: i64,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
        let client = pool.get().await?;
        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users ORDER BY created_at DESC LIMIT $1",
                &[&limit],
            )
            .await?;
        
        Ok(rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                username: r.get("username"),
                email: r.get("email"),
                first_name: r.get("first_name"),
                last_name: r.get("last_name"),
                age: r.get("age"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn pooled_cleanup(pool: &Pool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = pool.get().await?;
        client
            .execute(
                "DELETE FROM users WHERE username LIKE 'bench_user_%'",
                &[],
            )
            .await?;
        Ok(())
    }
}
