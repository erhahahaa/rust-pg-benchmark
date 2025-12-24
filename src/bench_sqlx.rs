//! SQLx benchmark implementation

use crate::{Comment, NewComment, NewPost, NewUser, Post, User, DATABASE_URL};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use uuid::Uuid;

pub struct SqlxBench;

impl SqlxBench {
    pub async fn connect() -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(10)
            .connect(DATABASE_URL)
            .await
    }
    
    /// Connect with a specific pool size for concurrent benchmarks
    pub async fn connect_with_pool_size(pool_size: u32) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(DATABASE_URL)
            .await
    }
    
    pub async fn insert_user(pool: &PgPool, user: &NewUser) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO users (username, email, first_name, last_name, age) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id"
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.age)
        .fetch_one(pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    pub async fn insert_users_batch(pool: &PgPool, users: &[NewUser]) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut ids = Vec::with_capacity(users.len());
        
        for user in users {
            let id = Self::insert_user(pool, user).await?;
            ids.push(id);
        }
        
        Ok(ids)
    }
    
    pub async fn select_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
             FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
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
    
    pub async fn select_users_limit(pool: &PgPool, limit: i64) -> Result<Vec<User>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
             FROM users ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
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
        pool: &PgPool,
        min_age: i32,
        max_age: i32,
        limit: i64,
    ) -> Result<Vec<User>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
             FROM users 
             WHERE age >= $1 AND age <= $2 
             ORDER BY age, username 
             LIMIT $3"
        )
        .bind(min_age)
        .bind(max_age)
        .bind(limit)
        .fetch_all(pool)
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
        pool: &PgPool,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE users SET first_name = $1, last_name = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(first_name)
        .bind(last_name)
        .bind(id)
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn insert_post(pool: &PgPool, post: &NewPost) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO posts (user_id, title, content, status) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id"
        )
        .bind(post.user_id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.status)
        .fetch_one(pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    pub async fn select_posts_with_user(
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<(Post, User)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT 
                p.id as post_id, p.user_id, p.title, p.content, p.status, p.view_count,
                p.created_at as post_created_at, p.updated_at as post_updated_at,
                u.id as user_id, u.username, u.email, u.first_name, u.last_name, u.age,
                u.created_at as user_created_at, u.updated_at as user_updated_at
             FROM posts p
             JOIN users u ON p.user_id = u.id
             ORDER BY p.created_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
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
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<(User, Post, Comment)>, sqlx::Error> {
        let rows = sqlx::query(
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
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
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
    
    pub async fn count_posts_per_user(pool: &PgPool) -> Result<Vec<(Uuid, i64)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT u.id, COUNT(p.id) as post_count
             FROM users u
             LEFT JOIN posts p ON u.id = p.user_id
             GROUP BY u.id
             ORDER BY post_count DESC"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }
    
    pub async fn insert_user_with_posts(
        pool: &PgPool,
        user: &NewUser,
        posts: &[NewPost],
    ) -> Result<Uuid, sqlx::Error> {
        let mut tx = pool.begin().await?;
        
        let row = sqlx::query(
            "INSERT INTO users (username, email, first_name, last_name, age) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id"
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.age)
        .fetch_one(&mut *tx)
        .await?;
        
        let user_id: Uuid = row.get("id");
        
        for post in posts {
            sqlx::query(
                "INSERT INTO posts (user_id, title, content, status) 
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(user_id)
            .bind(&post.title)
            .bind(&post.content)
            .bind(&post.status)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(user_id)
    }
    
    pub async fn cleanup(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE username LIKE 'bench_user_%'")
            .execute(pool)
            .await?;
        Ok(())
    }
    
    // Additional methods for heavy workload benchmarks
    
    pub async fn insert_comment(pool: &PgPool, comment: &NewComment) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO comments (post_id, user_id, content) 
             VALUES ($1, $2, $3) 
             RETURNING id"
        )
        .bind(comment.post_id)
        .bind(comment.user_id)
        .bind(&comment.content)
        .fetch_one(pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    pub async fn select_posts_by_status(
        pool: &PgPool,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Post>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, user_id, title, content, status, view_count, created_at, updated_at 
             FROM posts 
             WHERE status = $1 
             ORDER BY created_at DESC 
             LIMIT $2"
        )
        .bind(status)
        .bind(limit)
        .fetch_all(pool)
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
    
    pub async fn increment_view_count(pool: &PgPool, post_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
            .bind(post_id)
            .execute(pool)
            .await?;
        Ok(())
    }
    
    pub async fn search_users_by_name(
        pool: &PgPool,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<User>, sqlx::Error> {
        let pattern = format!("%{}%", pattern);
        let rows = sqlx::query(
            "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
             FROM users 
             WHERE first_name ILIKE $1 OR last_name ILIKE $1 
             ORDER BY username 
             LIMIT $2"
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(pool)
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
