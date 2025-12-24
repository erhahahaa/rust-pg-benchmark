//! Clorinde benchmark wrapper
//!
//! This module wraps the generated Clorinde queries for benchmarking.

use crate::{Comment, NewComment, NewPost, NewUser, Post, User, DATABASE_URL};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub use clorinde_queries::queries;
pub use clorinde_queries::prepared::PreparedStatements;

pub struct ClorindeBench;

impl ClorindeBench {
    pub async fn connect() -> Result<Client, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(client)
    }

    pub async fn prepare(client: &Client) -> Result<PreparedStatements, tokio_postgres::Error> {
        PreparedStatements::new(client).await
    }

    // Non-prepared statement versions (for fair comparison with unprepared queries)

    pub async fn insert_user(client: &Client, user: &NewUser) -> Result<Uuid, tokio_postgres::Error> {
        queries::insert_user(
            client,
            &user.username,
            &user.email,
            &user.first_name,
            &user.last_name,
            user.age,
        )
        .await
    }

    pub async fn insert_users_batch(
        client: &Client,
        users: &[NewUser],
    ) -> Result<Vec<Uuid>, tokio_postgres::Error> {
        let mut ids = Vec::with_capacity(users.len());
        for user in users {
            let id = Self::insert_user(client, user).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    pub async fn select_user_by_id(
        client: &Client,
        id: Uuid,
    ) -> Result<Option<User>, tokio_postgres::Error> {
        let user = queries::select_user_by_id(client, id).await?;
        Ok(user.map(|u| User {
            id: u.id,
            username: u.username,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            age: u.age,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }))
    }

    pub async fn select_users_limit(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<User>, tokio_postgres::Error> {
        let users = queries::select_users_limit(client, limit).await?;
        Ok(users
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at,
                updated_at: u.updated_at,
            })
            .collect())
    }

    pub async fn select_users_filtered(
        client: &Client,
        min_age: i32,
        max_age: i32,
        limit: i64,
    ) -> Result<Vec<User>, tokio_postgres::Error> {
        let users = queries::select_users_filtered(client, min_age, max_age, limit).await?;
        Ok(users
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at,
                updated_at: u.updated_at,
            })
            .collect())
    }

    pub async fn update_user(
        client: &Client,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<bool, tokio_postgres::Error> {
        let rows = queries::update_user(client, id, first_name, last_name).await?;
        Ok(rows > 0)
    }

    pub async fn delete_user(client: &Client, id: Uuid) -> Result<bool, tokio_postgres::Error> {
        let rows = queries::delete_user(client, id).await?;
        Ok(rows > 0)
    }

    pub async fn insert_post(client: &Client, post: &NewPost) -> Result<Uuid, tokio_postgres::Error> {
        queries::insert_post(client, post.user_id, &post.title, &post.content, &post.status).await
    }

    pub async fn select_posts_with_user(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<(Post, User)>, tokio_postgres::Error> {
        let results = queries::select_posts_with_user(client, limit).await?;
        Ok(results
            .into_iter()
            .map(|r| {
                (
                    Post {
                        id: r.post_id,
                        user_id: r.user_id,
                        title: r.title,
                        content: r.content,
                        status: r.status,
                        view_count: r.view_count,
                        created_at: r.post_created_at,
                        updated_at: r.post_updated_at,
                    },
                    User {
                        id: r.user_id,
                        username: r.username,
                        email: r.email,
                        first_name: r.first_name,
                        last_name: r.last_name,
                        age: r.age,
                        created_at: r.user_created_at,
                        updated_at: r.user_updated_at,
                    },
                )
            })
            .collect())
    }

    pub async fn select_users_posts_comments(
        client: &Client,
        limit: i64,
    ) -> Result<Vec<(User, Post, Comment)>, tokio_postgres::Error> {
        let results = queries::select_users_posts_comments(client, limit).await?;
        Ok(results
            .into_iter()
            .map(|r| {
                (
                    User {
                        id: r.user_id,
                        username: r.username,
                        email: r.email,
                        first_name: r.first_name,
                        last_name: r.last_name,
                        age: r.age,
                        created_at: r.user_created_at,
                        updated_at: r.user_updated_at,
                    },
                    Post {
                        id: r.post_id,
                        user_id: r.user_id,
                        title: r.title,
                        content: r.content,
                        status: r.status,
                        view_count: r.view_count,
                        created_at: r.post_created_at,
                        updated_at: r.post_updated_at,
                    },
                    Comment {
                        id: r.comment_id,
                        post_id: r.post_id,
                        user_id: r.user_id,
                        content: r.comment_content,
                        created_at: r.comment_created_at,
                    },
                )
            })
            .collect())
    }

    pub async fn count_posts_per_user(
        client: &Client,
    ) -> Result<Vec<(Uuid, i64)>, tokio_postgres::Error> {
        let results = queries::count_posts_per_user(client).await?;
        Ok(results
            .into_iter()
            .map(|r| (r.user_id, r.post_count))
            .collect())
    }

    pub async fn insert_user_with_posts(
        client: &Client,
        user: &NewUser,
        posts: &[NewPost],
    ) -> Result<Uuid, tokio_postgres::Error> {
        let user_id = Self::insert_user(client, user).await?;

        for post in posts {
            let mut post = post.clone();
            post.user_id = user_id;
            Self::insert_post(client, &post).await?;
        }

        Ok(user_id)
    }

    pub async fn cleanup(client: &Client) -> Result<(), tokio_postgres::Error> {
        queries::cleanup(client).await?;
        Ok(())
    }

    // Additional methods for heavy workload benchmarks

    pub async fn insert_comment(
        client: &Client,
        comment: &NewComment,
    ) -> Result<Uuid, tokio_postgres::Error> {
        queries::insert_comment(client, comment.post_id, comment.user_id, &comment.content).await
    }

    pub async fn select_posts_by_status(
        client: &Client,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Post>, tokio_postgres::Error> {
        let posts = queries::select_posts_by_status(client, status, limit).await?;
        Ok(posts
            .into_iter()
            .map(|p| Post {
                id: p.id,
                user_id: p.user_id,
                title: p.title,
                content: p.content,
                status: p.status,
                view_count: p.view_count,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect())
    }

    pub async fn increment_view_count(
        client: &Client,
        post_id: Uuid,
    ) -> Result<(), tokio_postgres::Error> {
        queries::increment_view_count(client, post_id).await?;
        Ok(())
    }

    pub async fn search_users_by_name(
        client: &Client,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<User>, tokio_postgres::Error> {
        let users = queries::search_users_by_name(client, pattern, limit).await?;
        Ok(users
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at,
                updated_at: u.updated_at,
            })
            .collect())
    }
}
