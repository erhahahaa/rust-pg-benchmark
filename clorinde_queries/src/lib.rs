//! Generated-style queries for Clorinde benchmark
//!
//! This module simulates what Clorinde would generate from SQL queries.
//! In a real project, you would use `clorinde` CLI to generate this code.

use chrono::{DateTime, Utc};
use tokio_postgres::{Client, Error, Row};
use uuid::Uuid;

/// User row from database
#[derive(Debug, Clone)]
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

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            age: row.get("age"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Post row from database
#[derive(Debug, Clone)]
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

/// Comment row from database
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Post with user join result
#[derive(Debug, Clone)]
pub struct PostWithUser {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: String,
    pub view_count: i32,
    pub post_created_at: Option<DateTime<Utc>>,
    pub post_updated_at: Option<DateTime<Utc>>,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: Option<i32>,
    pub user_created_at: Option<DateTime<Utc>>,
    pub user_updated_at: Option<DateTime<Utc>>,
}

impl From<&Row> for PostWithUser {
    fn from(row: &Row) -> Self {
        Self {
            post_id: row.get("post_id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            content: row.get("content"),
            status: row.get("status"),
            view_count: row.get("view_count"),
            post_created_at: row.get("post_created_at"),
            post_updated_at: row.get("post_updated_at"),
            username: row.get("username"),
            email: row.get("email"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            age: row.get("age"),
            user_created_at: row.get("user_created_at"),
            user_updated_at: row.get("user_updated_at"),
        }
    }
}

/// User post comment join result
#[derive(Debug, Clone)]
pub struct UserPostComment {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: Option<i32>,
    pub user_created_at: Option<DateTime<Utc>>,
    pub user_updated_at: Option<DateTime<Utc>>,
    pub post_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: String,
    pub view_count: i32,
    pub post_created_at: Option<DateTime<Utc>>,
    pub post_updated_at: Option<DateTime<Utc>>,
    pub comment_id: Uuid,
    pub comment_content: String,
    pub comment_created_at: Option<DateTime<Utc>>,
}

impl From<&Row> for UserPostComment {
    fn from(row: &Row) -> Self {
        Self {
            user_id: row.get("user_id"),
            username: row.get("username"),
            email: row.get("email"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            age: row.get("age"),
            user_created_at: row.get("user_created_at"),
            user_updated_at: row.get("user_updated_at"),
            post_id: row.get("post_id"),
            title: row.get("title"),
            content: row.get("content"),
            status: row.get("status"),
            view_count: row.get("view_count"),
            post_created_at: row.get("post_created_at"),
            post_updated_at: row.get("post_updated_at"),
            comment_id: row.get("comment_id"),
            comment_content: row.get("comment_content"),
            comment_created_at: row.get("comment_created_at"),
        }
    }
}

/// Post count per user
#[derive(Debug, Clone)]
pub struct UserPostCount {
    pub user_id: Uuid,
    pub post_count: i64,
}

impl From<&Row> for UserPostCount {
    fn from(row: &Row) -> Self {
        Self {
            user_id: row.get(0),
            post_count: row.get(1),
        }
    }
}

// ============================================================================
// Prepared statement holders - simulating Clorinde's generated code
// ============================================================================

pub mod queries {
    use super::*;

    /// Insert a new user
    pub async fn insert_user(
        client: &Client,
        username: &str,
        email: &str,
        first_name: &str,
        last_name: &str,
        age: Option<i32>,
    ) -> Result<Uuid, Error> {
        let row = client
            .query_one(
                "INSERT INTO users (username, email, first_name, last_name, age) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id",
                &[&username, &email, &first_name, &last_name, &age],
            )
            .await?;
        Ok(row.get("id"))
    }

    /// Select user by ID
    pub async fn select_user_by_id(client: &Client, id: Uuid) -> Result<Option<User>, Error> {
        let row = client
            .query_opt(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users WHERE id = $1",
                &[&id],
            )
            .await?;
        Ok(row.as_ref().map(User::from))
    }

    /// Select users with limit
    pub async fn select_users_limit(client: &Client, limit: i64) -> Result<Vec<User>, Error> {
        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                 FROM users ORDER BY created_at DESC LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(User::from).collect())
    }

    /// Select users with age filter
    pub async fn select_users_filtered(
        client: &Client,
        min_age: i32,
        max_age: i32,
        limit: i64,
    ) -> Result<Vec<User>, Error> {
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
        Ok(rows.iter().map(User::from).collect())
    }

    /// Update user
    pub async fn update_user(
        client: &Client,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<u64, Error> {
        client
            .execute(
                "UPDATE users SET first_name = $1, last_name = $2, updated_at = NOW() WHERE id = $3",
                &[&first_name, &last_name, &id],
            )
            .await
    }

    /// Delete user
    pub async fn delete_user(client: &Client, id: Uuid) -> Result<u64, Error> {
        client
            .execute("DELETE FROM users WHERE id = $1", &[&id])
            .await
    }

    /// Insert post
    pub async fn insert_post(
        client: &Client,
        user_id: Uuid,
        title: &str,
        content: &str,
        status: &str,
    ) -> Result<Uuid, Error> {
        let row = client
            .query_one(
                "INSERT INTO posts (user_id, title, content, status) 
                 VALUES ($1, $2, $3, $4) 
                 RETURNING id",
                &[&user_id, &title, &content, &status],
            )
            .await?;
        Ok(row.get("id"))
    }

    /// Select posts with user join
    pub async fn select_posts_with_user(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<PostWithUser>, Error> {
        let rows = client
            .query(
                "SELECT 
                    p.id as post_id, p.user_id, p.title, p.content, p.status, p.view_count,
                    p.created_at as post_created_at, p.updated_at as post_updated_at,
                    u.username, u.email, u.first_name, u.last_name, u.age,
                    u.created_at as user_created_at, u.updated_at as user_updated_at
                 FROM posts p
                 JOIN users u ON p.user_id = u.id
                 ORDER BY p.created_at DESC
                 LIMIT $1",
                &[&limit],
            )
            .await?;
        Ok(rows.iter().map(PostWithUser::from).collect())
    }

    /// Select users with posts and comments (triple join)
    pub async fn select_users_posts_comments(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<UserPostComment>, Error> {
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
        Ok(rows.iter().map(UserPostComment::from).collect())
    }

    /// Count posts per user
    pub async fn count_posts_per_user(client: &Client) -> Result<Vec<UserPostCount>, Error> {
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
        Ok(rows.iter().map(UserPostCount::from).collect())
    }

    /// Insert comment
    pub async fn insert_comment(
        client: &Client,
        post_id: Uuid,
        user_id: Uuid,
        content: &str,
    ) -> Result<Uuid, Error> {
        let row = client
            .query_one(
                "INSERT INTO comments (post_id, user_id, content) 
                 VALUES ($1, $2, $3) 
                 RETURNING id",
                &[&post_id, &user_id, &content],
            )
            .await?;
        Ok(row.get("id"))
    }

    /// Select posts by status
    pub async fn select_posts_by_status(
        client: &Client,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Post>, Error> {
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
            .map(|row| Post {
                id: row.get("id"),
                user_id: row.get("user_id"),
                title: row.get("title"),
                content: row.get("content"),
                status: row.get("status"),
                view_count: row.get("view_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    /// Increment view count
    pub async fn increment_view_count(client: &Client, post_id: Uuid) -> Result<u64, Error> {
        client
            .execute(
                "UPDATE posts SET view_count = view_count + 1 WHERE id = $1",
                &[&post_id],
            )
            .await
    }

    /// Search users by name
    pub async fn search_users_by_name(
        client: &Client,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<User>, Error> {
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
        Ok(rows.iter().map(User::from).collect())
    }

    /// Cleanup benchmark data
    pub async fn cleanup(client: &Client) -> Result<u64, Error> {
        client
            .execute("DELETE FROM users WHERE username LIKE 'bench_user_%'", &[])
            .await
    }
}

// ============================================================================
// Prepared statements version - more efficient for repeated queries
// ============================================================================

pub mod prepared {
    use super::*;
    use tokio_postgres::Statement;

    /// Prepared statement cache for optimal performance
    pub struct PreparedStatements {
        pub insert_user: Statement,
        pub select_user_by_id: Statement,
        pub select_users_limit: Statement,
        pub select_users_filtered: Statement,
        pub update_user: Statement,
        pub delete_user: Statement,
        pub insert_post: Statement,
        pub select_posts_with_user: Statement,
        pub select_users_posts_comments: Statement,
        pub count_posts_per_user: Statement,
        pub insert_comment: Statement,
        pub select_posts_by_status: Statement,
        pub increment_view_count: Statement,
        pub search_users_by_name: Statement,
        pub cleanup: Statement,
    }

    impl PreparedStatements {
        pub async fn new(client: &Client) -> Result<Self, Error> {
            Ok(Self {
                insert_user: client
                    .prepare(
                        "INSERT INTO users (username, email, first_name, last_name, age) 
                         VALUES ($1, $2, $3, $4, $5) 
                         RETURNING id",
                    )
                    .await?,
                select_user_by_id: client
                    .prepare(
                        "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                         FROM users WHERE id = $1",
                    )
                    .await?,
                select_users_limit: client
                    .prepare(
                        "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                         FROM users ORDER BY created_at DESC LIMIT $1",
                    )
                    .await?,
                select_users_filtered: client
                    .prepare(
                        "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                         FROM users 
                         WHERE age >= $1 AND age <= $2 
                         ORDER BY age, username 
                         LIMIT $3",
                    )
                    .await?,
                update_user: client
                    .prepare(
                        "UPDATE users SET first_name = $1, last_name = $2, updated_at = NOW() WHERE id = $3",
                    )
                    .await?,
                delete_user: client.prepare("DELETE FROM users WHERE id = $1").await?,
                insert_post: client
                    .prepare(
                        "INSERT INTO posts (user_id, title, content, status) 
                         VALUES ($1, $2, $3, $4) 
                         RETURNING id",
                    )
                    .await?,
                select_posts_with_user: client
                    .prepare(
                        "SELECT 
                            p.id as post_id, p.user_id, p.title, p.content, p.status, p.view_count,
                            p.created_at as post_created_at, p.updated_at as post_updated_at,
                            u.username, u.email, u.first_name, u.last_name, u.age,
                            u.created_at as user_created_at, u.updated_at as user_updated_at
                         FROM posts p
                         JOIN users u ON p.user_id = u.id
                         ORDER BY p.created_at DESC
                         LIMIT $1",
                    )
                    .await?,
                select_users_posts_comments: client
                    .prepare(
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
                    )
                    .await?,
                count_posts_per_user: client
                    .prepare(
                        "SELECT u.id, COUNT(p.id) as post_count
                         FROM users u
                         LEFT JOIN posts p ON u.id = p.user_id
                         GROUP BY u.id
                         ORDER BY post_count DESC",
                    )
                    .await?,
                insert_comment: client
                    .prepare(
                        "INSERT INTO comments (post_id, user_id, content) 
                         VALUES ($1, $2, $3) 
                         RETURNING id",
                    )
                    .await?,
                select_posts_by_status: client
                    .prepare(
                        "SELECT id, user_id, title, content, status, view_count, created_at, updated_at 
                         FROM posts 
                         WHERE status = $1 
                         ORDER BY created_at DESC 
                         LIMIT $2",
                    )
                    .await?,
                increment_view_count: client
                    .prepare("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
                    .await?,
                search_users_by_name: client
                    .prepare(
                        "SELECT id, username, email, first_name, last_name, age, created_at, updated_at 
                         FROM users 
                         WHERE first_name ILIKE $1 OR last_name ILIKE $1 
                         ORDER BY username 
                         LIMIT $2",
                    )
                    .await?,
                cleanup: client
                    .prepare("DELETE FROM users WHERE username LIKE 'bench_user_%'")
                    .await?,
            })
        }

        pub async fn insert_user(
            &self,
            client: &Client,
            username: &str,
            email: &str,
            first_name: &str,
            last_name: &str,
            age: Option<i32>,
        ) -> Result<Uuid, Error> {
            let row = client
                .query_one(
                    &self.insert_user,
                    &[&username, &email, &first_name, &last_name, &age],
                )
                .await?;
            Ok(row.get("id"))
        }

        pub async fn select_user_by_id(
            &self,
            client: &Client,
            id: Uuid,
        ) -> Result<Option<User>, Error> {
            let row = client.query_opt(&self.select_user_by_id, &[&id]).await?;
            Ok(row.as_ref().map(User::from))
        }

        pub async fn select_users_limit(
            &self,
            client: &Client,
            limit: i64,
        ) -> Result<Vec<User>, Error> {
            let rows = client.query(&self.select_users_limit, &[&limit]).await?;
            Ok(rows.iter().map(User::from).collect())
        }

        pub async fn select_users_filtered(
            &self,
            client: &Client,
            min_age: i32,
            max_age: i32,
            limit: i64,
        ) -> Result<Vec<User>, Error> {
            let rows = client
                .query(&self.select_users_filtered, &[&min_age, &max_age, &limit])
                .await?;
            Ok(rows.iter().map(User::from).collect())
        }

        pub async fn update_user(
            &self,
            client: &Client,
            id: Uuid,
            first_name: &str,
            last_name: &str,
        ) -> Result<u64, Error> {
            client
                .execute(&self.update_user, &[&first_name, &last_name, &id])
                .await
        }

        pub async fn select_posts_with_user(
            &self,
            client: &Client,
            limit: i64,
        ) -> Result<Vec<PostWithUser>, Error> {
            let rows = client
                .query(&self.select_posts_with_user, &[&limit])
                .await?;
            Ok(rows.iter().map(PostWithUser::from).collect())
        }

        pub async fn select_users_posts_comments(
            &self,
            client: &Client,
            limit: i64,
        ) -> Result<Vec<UserPostComment>, Error> {
            let rows = client
                .query(&self.select_users_posts_comments, &[&limit])
                .await?;
            Ok(rows.iter().map(UserPostComment::from).collect())
        }

        pub async fn count_posts_per_user(
            &self,
            client: &Client,
        ) -> Result<Vec<UserPostCount>, Error> {
            let rows = client.query(&self.count_posts_per_user, &[]).await?;
            Ok(rows.iter().map(UserPostCount::from).collect())
        }

        pub async fn insert_post(
            &self,
            client: &Client,
            user_id: Uuid,
            title: &str,
            content: &str,
            status: &str,
        ) -> Result<Uuid, Error> {
            let row = client
                .query_one(&self.insert_post, &[&user_id, &title, &content, &status])
                .await?;
            Ok(row.get("id"))
        }

        pub async fn cleanup(&self, client: &Client) -> Result<u64, Error> {
            client.execute(&self.cleanup, &[]).await
        }
    }
}
