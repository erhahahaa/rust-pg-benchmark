//! Diesel benchmark implementation

use crate::{Comment, NewComment, NewPost, NewUser, Post, User, DATABASE_URL};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use uuid::Uuid;

// Diesel schema
pub mod schema {
    diesel::table! {
        users (id) {
            id -> Uuid,
            username -> Varchar,
            email -> Varchar,
            first_name -> Varchar,
            last_name -> Varchar,
            age -> Nullable<Int4>,
            created_at -> Nullable<Timestamptz>,
            updated_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        posts (id) {
            id -> Uuid,
            user_id -> Uuid,
            title -> Varchar,
            content -> Text,
            status -> Varchar,
            view_count -> Int4,
            created_at -> Nullable<Timestamptz>,
            updated_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        comments (id) {
            id -> Uuid,
            post_id -> Uuid,
            user_id -> Uuid,
            content -> Text,
            created_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        tags (id) {
            id -> Uuid,
            name -> Varchar,
            color -> Varchar,
            created_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        post_tags (post_id, tag_id) {
            post_id -> Uuid,
            tag_id -> Uuid,
        }
    }

    diesel::joinable!(posts -> users (user_id));
    diesel::joinable!(comments -> posts (post_id));
    diesel::joinable!(comments -> users (user_id));
    diesel::joinable!(post_tags -> posts (post_id));
    diesel::joinable!(post_tags -> tags (tag_id));

    diesel::allow_tables_to_appear_in_same_query!(users, posts, comments, tags, post_tags,);
}

use schema::*;

// Diesel models
#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = users)]
pub struct DieselUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: Option<i32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct DieselNewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: Option<i32>,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = posts)]
pub struct DieselPost {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: String,
    pub view_count: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct DieselNewPost<'a> {
    pub user_id: Uuid,
    pub title: &'a str,
    pub content: &'a str,
    pub status: &'a str,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = comments)]
pub struct DieselComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = comments)]
pub struct DieselNewComment<'a> {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: &'a str,
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DieselBench;

impl DieselBench {
    pub fn connect() -> Result<DbPool, diesel::r2d2::PoolError> {
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
        Pool::builder().max_size(10).build(manager)
    }

    /// Connect with a specific pool size for concurrent benchmarks
    pub fn connect_with_pool_size(pool_size: u32) -> Result<DbPool, diesel::r2d2::PoolError> {
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
        Pool::builder().max_size(pool_size).build(manager)
    }

    pub fn insert_user(conn: &mut PgConnection, user: &NewUser) -> Result<Uuid, diesel::result::Error> {
        let new_user = DieselNewUser {
            username: &user.username,
            email: &user.email,
            first_name: &user.first_name,
            last_name: &user.last_name,
            age: user.age,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(users::id)
            .get_result(conn)
    }

    pub fn insert_users_batch(
        conn: &mut PgConnection,
        users_data: &[NewUser],
    ) -> Result<Vec<Uuid>, diesel::result::Error> {
        let new_users: Vec<DieselNewUser> = users_data
            .iter()
            .map(|u| DieselNewUser {
                username: &u.username,
                email: &u.email,
                first_name: &u.first_name,
                last_name: &u.last_name,
                age: u.age,
            })
            .collect();

        diesel::insert_into(users::table)
            .values(&new_users)
            .returning(users::id)
            .get_results(conn)
    }

    pub fn select_user_by_id(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<Option<User>, diesel::result::Error> {
        let user = users::table
            .find(id)
            .select(DieselUser::as_select())
            .first(conn)
            .optional()?;

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

    pub fn select_users_limit(
        conn: &mut PgConnection,
        limit: i64,
    ) -> Result<Vec<User>, diesel::result::Error> {
        let users_list = users::table
            .order(users::created_at.desc())
            .limit(limit)
            .select(DieselUser::as_select())
            .load(conn)?;

        Ok(users_list
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

    pub fn select_users_filtered(
        conn: &mut PgConnection,
        min_age: i32,
        max_age: i32,
        limit: i64,
    ) -> Result<Vec<User>, diesel::result::Error> {
        let users_list = users::table
            .filter(users::age.ge(min_age))
            .filter(users::age.le(max_age))
            .order((users::age.asc(), users::username.asc()))
            .limit(limit)
            .select(DieselUser::as_select())
            .load(conn)?;

        Ok(users_list
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

    pub fn update_user(
        conn: &mut PgConnection,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<bool, diesel::result::Error> {
        let rows_affected = diesel::update(users::table.find(id))
            .set((
                users::first_name.eq(first_name),
                users::last_name.eq(last_name),
                users::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)?;

        Ok(rows_affected > 0)
    }

    pub fn delete_user(conn: &mut PgConnection, id: Uuid) -> Result<bool, diesel::result::Error> {
        let rows_affected = diesel::delete(users::table.find(id)).execute(conn)?;
        Ok(rows_affected > 0)
    }

    pub fn insert_post(conn: &mut PgConnection, post: &NewPost) -> Result<Uuid, diesel::result::Error> {
        let new_post = DieselNewPost {
            user_id: post.user_id,
            title: &post.title,
            content: &post.content,
            status: &post.status,
        };

        diesel::insert_into(posts::table)
            .values(&new_post)
            .returning(posts::id)
            .get_result(conn)
    }

    pub fn select_posts_with_user(
        conn: &mut PgConnection,
        limit: i64,
    ) -> Result<Vec<(Post, User)>, diesel::result::Error> {
        let results = posts::table
            .inner_join(users::table)
            .order(posts::created_at.desc())
            .limit(limit)
            .select((DieselPost::as_select(), DieselUser::as_select()))
            .load::<(DieselPost, DieselUser)>(conn)?;

        Ok(results
            .into_iter()
            .map(|(p, u)| {
                (
                    Post {
                        id: p.id,
                        user_id: p.user_id,
                        title: p.title,
                        content: p.content,
                        status: p.status,
                        view_count: p.view_count,
                        created_at: p.created_at,
                        updated_at: p.updated_at,
                    },
                    User {
                        id: u.id,
                        username: u.username,
                        email: u.email,
                        first_name: u.first_name,
                        last_name: u.last_name,
                        age: u.age,
                        created_at: u.created_at,
                        updated_at: u.updated_at,
                    },
                )
            })
            .collect())
    }

    pub fn select_users_posts_comments(
        conn: &mut PgConnection,
        limit: i64,
    ) -> Result<Vec<(User, Post, Comment)>, diesel::result::Error> {
        let results = comments::table
            .inner_join(posts::table.inner_join(users::table))
            .order((
                users::created_at.desc(),
                posts::created_at.desc(),
                comments::created_at.desc(),
            ))
            .limit(limit)
            .select((
                DieselUser::as_select(),
                DieselPost::as_select(),
                DieselComment::as_select(),
            ))
            .load::<(DieselUser, DieselPost, DieselComment)>(conn)?;

        Ok(results
            .into_iter()
            .map(|(u, p, c)| {
                (
                    User {
                        id: u.id,
                        username: u.username,
                        email: u.email,
                        first_name: u.first_name,
                        last_name: u.last_name,
                        age: u.age,
                        created_at: u.created_at,
                        updated_at: u.updated_at,
                    },
                    Post {
                        id: p.id,
                        user_id: p.user_id,
                        title: p.title,
                        content: p.content,
                        status: p.status,
                        view_count: p.view_count,
                        created_at: p.created_at,
                        updated_at: p.updated_at,
                    },
                    Comment {
                        id: c.id,
                        post_id: c.post_id,
                        user_id: c.user_id,
                        content: c.content,
                        created_at: c.created_at,
                    },
                )
            })
            .collect())
    }

    pub fn count_posts_per_user(
        conn: &mut PgConnection,
    ) -> Result<Vec<(Uuid, i64)>, diesel::result::Error> {
        use diesel::dsl::count;

        users::table
            .left_join(posts::table)
            .group_by(users::id)
            .select((users::id, count(posts::id.nullable())))
            .order(count(posts::id.nullable()).desc())
            .load(conn)
    }

    pub fn insert_user_with_posts(
        conn: &mut PgConnection,
        user: &NewUser,
        posts_data: &[NewPost],
    ) -> Result<Uuid, diesel::result::Error> {
        conn.transaction(|conn| {
            let user_id = Self::insert_user(conn, user)?;

            for post in posts_data {
                let new_post = DieselNewPost {
                    user_id,
                    title: &post.title,
                    content: &post.content,
                    status: &post.status,
                };
                diesel::insert_into(posts::table)
                    .values(&new_post)
                    .execute(conn)?;
            }

            Ok(user_id)
        })
    }

    pub fn cleanup(conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        diesel::delete(users::table.filter(users::username.like("bench_user_%"))).execute(conn)?;
        Ok(())
    }

    // Additional methods for heavy workload benchmarks

    pub fn insert_comment(
        conn: &mut PgConnection,
        comment: &NewComment,
    ) -> Result<Uuid, diesel::result::Error> {
        let new_comment = DieselNewComment {
            post_id: comment.post_id,
            user_id: comment.user_id,
            content: &comment.content,
        };

        diesel::insert_into(comments::table)
            .values(&new_comment)
            .returning(comments::id)
            .get_result(conn)
    }

    pub fn select_posts_by_status(
        conn: &mut PgConnection,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Post>, diesel::result::Error> {
        let posts_list = posts::table
            .filter(posts::status.eq(status))
            .order(posts::created_at.desc())
            .limit(limit)
            .select(DieselPost::as_select())
            .load(conn)?;

        Ok(posts_list
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

    pub fn increment_view_count(
        conn: &mut PgConnection,
        post_id: Uuid,
    ) -> Result<(), diesel::result::Error> {
        diesel::update(posts::table.find(post_id))
            .set(posts::view_count.eq(posts::view_count + 1))
            .execute(conn)?;
        Ok(())
    }

    pub fn search_users_by_name(
        conn: &mut PgConnection,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<User>, diesel::result::Error> {
        let pattern = format!("%{}%", pattern);
        let users_list = users::table
            .filter(
                users::first_name
                    .ilike(&pattern)
                    .or(users::last_name.ilike(&pattern)),
            )
            .order(users::username.asc())
            .limit(limit)
            .select(DieselUser::as_select())
            .load(conn)?;

        Ok(users_list
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
