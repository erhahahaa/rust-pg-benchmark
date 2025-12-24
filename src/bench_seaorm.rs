//! SeaORM benchmark implementation

use crate::{Comment, NewComment, NewPost, NewUser, Post, User, DATABASE_URL};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use uuid::Uuid;

// Define SeaORM entities

pub mod users {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub username: String,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        pub age: Option<i32>,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::posts::Entity")]
        Posts,
        #[sea_orm(has_many = "super::comments::Entity")]
        Comments,
    }

    impl Related<super::posts::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Posts.def()
        }
    }

    impl Related<super::comments::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Comments.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod posts {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "posts")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub user_id: Uuid,
        pub title: String,
        pub content: String,
        pub status: String,
        pub view_count: i32,
        pub created_at: Option<DateTimeWithTimeZone>,
        pub updated_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::users::Entity",
            from = "Column::UserId",
            to = "super::users::Column::Id"
        )]
        User,
        #[sea_orm(has_many = "super::comments::Entity")]
        Comments,
    }

    impl Related<super::users::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }

    impl Related<super::comments::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Comments.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod comments {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "comments")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub post_id: Uuid,
        pub user_id: Uuid,
        pub content: String,
        pub created_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::posts::Entity",
            from = "Column::PostId",
            to = "super::posts::Column::Id"
        )]
        Post,
        #[sea_orm(
            belongs_to = "super::users::Entity",
            from = "Column::UserId",
            to = "super::users::Column::Id"
        )]
        User,
    }

    impl Related<super::posts::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Post.def()
        }
    }

    impl Related<super::users::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub struct SeaOrmBench;

impl SeaOrmBench {
    pub async fn connect() -> Result<DatabaseConnection, DbErr> {
        Database::connect(DATABASE_URL).await
    }

    /// Connect with a specific pool size for concurrent benchmarks
    pub async fn connect_with_pool_size(pool_size: u32) -> Result<DatabaseConnection, DbErr> {
        let mut opt = sea_orm::ConnectOptions::new(DATABASE_URL);
        opt.max_connections(pool_size);
        Database::connect(opt).await
    }

    pub async fn insert_user(db: &DatabaseConnection, user: &NewUser) -> Result<Uuid, DbErr> {
        let id = Uuid::new_v4();
        let model = users::ActiveModel {
            id: ActiveValue::Set(id),
            username: ActiveValue::Set(user.username.clone()),
            email: ActiveValue::Set(user.email.clone()),
            first_name: ActiveValue::Set(user.first_name.clone()),
            last_name: ActiveValue::Set(user.last_name.clone()),
            age: ActiveValue::Set(user.age),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        };

        model.insert(db).await?;
        Ok(id)
    }

    pub async fn insert_users_batch(
        db: &DatabaseConnection,
        users_data: &[NewUser],
    ) -> Result<Vec<Uuid>, DbErr> {
        let mut ids = Vec::with_capacity(users_data.len());

        for user in users_data {
            let id = Self::insert_user(db, user).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    pub async fn select_user_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<User>, DbErr> {
        let user = users::Entity::find_by_id(id).one(db).await?;

        Ok(user.map(|u| User {
            id: u.id,
            username: u.username,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            age: u.age,
            created_at: u.created_at.map(|dt| dt.into()),
            updated_at: u.updated_at.map(|dt| dt.into()),
        }))
    }

    pub async fn select_users_limit(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<User>, DbErr> {
        let users_list = users::Entity::find()
            .order_by_desc(users::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await?;

        Ok(users_list
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at.map(|dt| dt.into()),
                updated_at: u.updated_at.map(|dt| dt.into()),
            })
            .collect())
    }

    pub async fn select_users_filtered(
        db: &DatabaseConnection,
        min_age: i32,
        max_age: i32,
        limit: u64,
    ) -> Result<Vec<User>, DbErr> {
        let users_list = users::Entity::find()
            .filter(users::Column::Age.gte(min_age))
            .filter(users::Column::Age.lte(max_age))
            .order_by_asc(users::Column::Age)
            .order_by_asc(users::Column::Username)
            .limit(limit)
            .all(db)
            .await?;

        Ok(users_list
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at.map(|dt| dt.into()),
                updated_at: u.updated_at.map(|dt| dt.into()),
            })
            .collect())
    }

    pub async fn update_user(
        db: &DatabaseConnection,
        id: Uuid,
        first_name: &str,
        last_name: &str,
    ) -> Result<bool, DbErr> {
        let user = users::Entity::find_by_id(id).one(db).await?;

        if let Some(user) = user {
            let mut active: users::ActiveModel = user.into();
            active.first_name = ActiveValue::Set(first_name.to_string());
            active.last_name = ActiveValue::Set(last_name.to_string());
            active.update(db).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_user(db: &DatabaseConnection, id: Uuid) -> Result<bool, DbErr> {
        let result = users::Entity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn insert_post(db: &DatabaseConnection, post: &NewPost) -> Result<Uuid, DbErr> {
        let id = Uuid::new_v4();
        let model = posts::ActiveModel {
            id: ActiveValue::Set(id),
            user_id: ActiveValue::Set(post.user_id),
            title: ActiveValue::Set(post.title.clone()),
            content: ActiveValue::Set(post.content.clone()),
            status: ActiveValue::Set(post.status.clone()),
            view_count: ActiveValue::Set(0),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        };

        model.insert(db).await?;
        Ok(id)
    }

    pub async fn select_posts_with_user(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<(Post, User)>, DbErr> {
        let posts_with_users = posts::Entity::find()
            .find_also_related(users::Entity)
            .order_by_desc(posts::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await?;

        Ok(posts_with_users
            .into_iter()
            .filter_map(|(p, u)| {
                u.map(|u| {
                    (
                        Post {
                            id: p.id,
                            user_id: p.user_id,
                            title: p.title,
                            content: p.content,
                            status: p.status,
                            view_count: p.view_count,
                            created_at: p.created_at.map(|dt| dt.into()),
                            updated_at: p.updated_at.map(|dt| dt.into()),
                        },
                        User {
                            id: u.id,
                            username: u.username,
                            email: u.email,
                            first_name: u.first_name,
                            last_name: u.last_name,
                            age: u.age,
                            created_at: u.created_at.map(|dt| dt.into()),
                            updated_at: u.updated_at.map(|dt| dt.into()),
                        },
                    )
                })
            })
            .collect())
    }

    pub async fn select_users_posts_comments(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<(User, Post, Comment)>, DbErr> {
        // SeaORM doesn't have native triple join, so we do it with separate queries
        let comments_list = comments::Entity::find()
            .order_by_desc(comments::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await?;

        let mut results = Vec::new();
        for c in comments_list {
            if let Some(post) = posts::Entity::find_by_id(c.post_id).one(db).await? {
                if let Some(user) = users::Entity::find_by_id(post.user_id).one(db).await? {
                    results.push((
                        User {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            first_name: user.first_name,
                            last_name: user.last_name,
                            age: user.age,
                            created_at: user.created_at.map(|dt| dt.into()),
                            updated_at: user.updated_at.map(|dt| dt.into()),
                        },
                        Post {
                            id: post.id,
                            user_id: post.user_id,
                            title: post.title,
                            content: post.content,
                            status: post.status,
                            view_count: post.view_count,
                            created_at: post.created_at.map(|dt| dt.into()),
                            updated_at: post.updated_at.map(|dt| dt.into()),
                        },
                        Comment {
                            id: c.id,
                            post_id: c.post_id,
                            user_id: c.user_id,
                            content: c.content,
                            created_at: c.created_at.map(|dt| dt.into()),
                        },
                    ));
                }
            }
        }

        Ok(results)
    }

    pub async fn count_posts_per_user(
        db: &DatabaseConnection,
    ) -> Result<Vec<(Uuid, i64)>, DbErr> {
        // Use raw SQL for aggregate query as SeaORM's group by is complex
        let results: Vec<(Uuid, i64)> = db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT u.id, COUNT(p.id) as post_count
                 FROM users u
                 LEFT JOIN posts p ON u.id = p.user_id
                 GROUP BY u.id
                 ORDER BY post_count DESC".to_string(),
            ))
            .await?
            .into_iter()
            .filter_map(|row| {
                let id: Option<Uuid> = row.try_get("", "id").ok();
                let count: Option<i64> = row.try_get("", "post_count").ok();
                match (id, count) {
                    (Some(id), Some(count)) => Some((id, count)),
                    _ => None,
                }
            })
            .collect();

        Ok(results)
    }

    pub async fn insert_user_with_posts(
        db: &DatabaseConnection,
        user: &NewUser,
        posts_data: &[NewPost],
    ) -> Result<Uuid, DbErr> {
        let txn = db.begin().await?;

        let user_id = Uuid::new_v4();
        let user_model = users::ActiveModel {
            id: ActiveValue::Set(user_id),
            username: ActiveValue::Set(user.username.clone()),
            email: ActiveValue::Set(user.email.clone()),
            first_name: ActiveValue::Set(user.first_name.clone()),
            last_name: ActiveValue::Set(user.last_name.clone()),
            age: ActiveValue::Set(user.age),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        };
        user_model.insert(&txn).await?;

        for post in posts_data {
            let post_model = posts::ActiveModel {
                id: ActiveValue::Set(Uuid::new_v4()),
                user_id: ActiveValue::Set(user_id),
                title: ActiveValue::Set(post.title.clone()),
                content: ActiveValue::Set(post.content.clone()),
                status: ActiveValue::Set(post.status.clone()),
                view_count: ActiveValue::Set(0),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::NotSet,
            };
            post_model.insert(&txn).await?;
        }

        txn.commit().await?;
        Ok(user_id)
    }

    pub async fn cleanup(db: &DatabaseConnection) -> Result<(), DbErr> {
        users::Entity::delete_many()
            .filter(users::Column::Username.starts_with("bench_user_"))
            .exec(db)
            .await?;
        Ok(())
    }

    // Additional methods for heavy workload benchmarks

    pub async fn insert_comment(
        db: &DatabaseConnection,
        comment: &NewComment,
    ) -> Result<Uuid, DbErr> {
        let id = Uuid::new_v4();
        let model = comments::ActiveModel {
            id: ActiveValue::Set(id),
            post_id: ActiveValue::Set(comment.post_id),
            user_id: ActiveValue::Set(comment.user_id),
            content: ActiveValue::Set(comment.content.clone()),
            created_at: ActiveValue::NotSet,
        };

        model.insert(db).await?;
        Ok(id)
    }

    pub async fn select_posts_by_status(
        db: &DatabaseConnection,
        status: &str,
        limit: u64,
    ) -> Result<Vec<Post>, DbErr> {
        let posts_list = posts::Entity::find()
            .filter(posts::Column::Status.eq(status))
            .order_by_desc(posts::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await?;

        Ok(posts_list
            .into_iter()
            .map(|p| Post {
                id: p.id,
                user_id: p.user_id,
                title: p.title,
                content: p.content,
                status: p.status,
                view_count: p.view_count,
                created_at: p.created_at.map(|dt| dt.into()),
                updated_at: p.updated_at.map(|dt| dt.into()),
            })
            .collect())
    }

    pub async fn increment_view_count(db: &DatabaseConnection, post_id: Uuid) -> Result<(), DbErr> {
        if let Some(post) = posts::Entity::find_by_id(post_id).one(db).await? {
            let mut active: posts::ActiveModel = post.into();
            active.view_count = ActiveValue::Set(
                active.view_count.unwrap() + 1
            );
            active.update(db).await?;
        }
        Ok(())
    }

    pub async fn search_users_by_name(
        db: &DatabaseConnection,
        pattern: &str,
        limit: u64,
    ) -> Result<Vec<User>, DbErr> {
        let pattern = format!("%{}%", pattern);
        let users_list = users::Entity::find()
            .filter(
                users::Column::FirstName
                    .contains(&pattern)
                    .or(users::Column::LastName.contains(&pattern)),
            )
            .order_by_asc(users::Column::Username)
            .limit(limit)
            .all(db)
            .await?;

        Ok(users_list
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                age: u.age,
                created_at: u.created_at.map(|dt| dt.into()),
                updated_at: u.updated_at.map(|dt| dt.into()),
            })
            .collect())
    }
}
