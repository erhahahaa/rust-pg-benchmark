//! Comprehensive PostgreSQL Library Benchmark
//!
//! This benchmark suite provides fair and comprehensive comparisons between:
//! - tokio-postgres: Low-level async PostgreSQL driver
//! - sqlx: Compile-time checked SQL queries
//! - sea-orm: Async ORM built on sqlx
//! - diesel: Synchronous ORM with type safety
//! - clorinde: Generated type-safe queries from SQL
//!
//! Benchmark Categories:
//! 1. Insert Operations (single and batch)
//! 2. Select Operations (simple and filtered)
//! 3. Update Operations
//! 4. Delete Operations
//! 5. Join Operations (single and multi-table)
//! 6. Aggregate Operations
//! 7. Transaction Operations
//! 8. Heavy Workload Simulation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pg_benchmark::{
    bench_clorinde::ClorindeBench,
    bench_diesel::DieselBench,
    bench_seaorm::SeaOrmBench,
    bench_sqlx::SqlxBench,
    bench_tokio_postgres::TokioPostgresBench,
    NewPost, NewUser,
};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

// Benchmark sizes
const SIZES: &[usize] = &[10, 100, 1000];

fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

// ============================================================================
// Insert Benchmarks
// ============================================================================

fn bench_insert_single(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("insert_single_user");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            counter += 1;
            let user = NewUser::generate(counter);
            rt.block_on(TokioPostgresBench::insert_user(&client, &user))
                .unwrap()
        });
        rt.block_on(TokioPostgresBench::cleanup(&client)).unwrap();
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            counter += 1;
            let user = NewUser::generate(counter);
            rt.block_on(SqlxBench::insert_user(&pool, &user)).unwrap()
        });
        rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            counter += 1;
            let user = NewUser::generate(counter);
            rt.block_on(SeaOrmBench::insert_user(&db, &user)).unwrap()
        });
        rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
    });

    // diesel (sync)
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            counter += 1;
            let user = NewUser::generate(counter);
            DieselBench::insert_user(&mut conn, &user).unwrap()
        });
        DieselBench::cleanup(&mut conn).unwrap();
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            counter += 1;
            let user = NewUser::generate(counter);
            rt.block_on(ClorindeBench::insert_user(&client, &user))
                .unwrap()
        });
        rt.block_on(ClorindeBench::cleanup(&client)).unwrap();
    });

    group.finish();
}

fn bench_insert_batch(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("insert_batch_users");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    for size in SIZES {
        let users: Vec<NewUser> = (0..*size).map(|i| NewUser::generate(i)).collect();

        group.throughput(Throughput::Elements(*size as u64));

        // tokio-postgres
        group.bench_with_input(BenchmarkId::new("tokio_postgres", size), size, |b, _| {
            let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(TokioPostgresBench::insert_users_batch(&client, &users))
                    .unwrap()
            });
            rt.block_on(TokioPostgresBench::cleanup(&client)).unwrap();
        });

        // sqlx
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SqlxBench::insert_users_batch(&pool, &users))
                    .unwrap()
            });
            rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
        });

        // sea-orm
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SeaOrmBench::insert_users_batch(&db, &users))
                    .unwrap()
            });
            rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            b.iter(|| DieselBench::insert_users_batch(&mut conn, &users).unwrap());
            DieselBench::cleanup(&mut conn).unwrap();
        });

        // clorinde
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(ClorindeBench::insert_users_batch(&client, &users))
                    .unwrap()
            });
            rt.block_on(ClorindeBench::cleanup(&client)).unwrap();
        });
    }

    group.finish();
}

// ============================================================================
// Select Benchmarks
// ============================================================================

fn bench_select_limit(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("select_users_limit");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));

        let limit = *size as i64;

        // tokio-postgres
        group.bench_with_input(BenchmarkId::new("tokio_postgres", size), size, |b, _| {
            let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(TokioPostgresBench::select_users_limit(&client, limit))
                    .unwrap()
            });
        });

        // sqlx
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SqlxBench::select_users_limit(&pool, limit))
                    .unwrap()
            });
        });

        // sea-orm
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SeaOrmBench::select_users_limit(&db, *size as u64))
                    .unwrap()
            });
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            b.iter(|| DieselBench::select_users_limit(&mut conn, limit).unwrap());
        });

        // clorinde
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(ClorindeBench::select_users_limit(&client, limit))
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_select_filtered(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("select_users_filtered");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));

        let limit = *size as i64;
        let min_age = 25;
        let max_age = 55;

        // tokio-postgres
        group.bench_with_input(BenchmarkId::new("tokio_postgres", size), size, |b, _| {
            let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(TokioPostgresBench::select_users_filtered(
                    &client, min_age, max_age, limit,
                ))
                .unwrap()
            });
        });

        // sqlx
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SqlxBench::select_users_filtered(&pool, min_age, max_age, limit))
                    .unwrap()
            });
        });

        // sea-orm
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SeaOrmBench::select_users_filtered(
                    &db,
                    min_age,
                    max_age,
                    *size as u64,
                ))
                .unwrap()
            });
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            b.iter(|| {
                DieselBench::select_users_filtered(&mut conn, min_age, max_age, limit).unwrap()
            });
        });

        // clorinde
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(ClorindeBench::select_users_filtered(
                    &client, min_age, max_age, limit,
                ))
                .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_select_by_id(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("select_user_by_id");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(200);

    // Setup: get some user IDs
    let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
    let users = rt
        .block_on(TokioPostgresBench::select_users_limit(&client, 100))
        .unwrap();
    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(TokioPostgresBench::select_user_by_id(&client, id))
                .unwrap()
        });
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(SqlxBench::select_user_by_id(&pool, id)).unwrap()
        });
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(SeaOrmBench::select_user_by_id(&db, id)).unwrap()
        });
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            DieselBench::select_user_by_id(&mut conn, id).unwrap()
        });
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(ClorindeBench::select_user_by_id(&client, id))
                .unwrap()
        });
    });

    group.finish();
}

// ============================================================================
// Update Benchmarks
// ============================================================================

fn bench_update_user(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("update_user");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Setup: get some user IDs
    let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
    let users = rt
        .block_on(TokioPostgresBench::select_users_limit(&client, 100))
        .unwrap();
    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(TokioPostgresBench::update_user(
                &client,
                id,
                "UpdatedFirst",
                "UpdatedLast",
            ))
            .unwrap()
        });
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(SqlxBench::update_user(&pool, id, "UpdatedFirst", "UpdatedLast"))
                .unwrap()
        });
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(SeaOrmBench::update_user(&db, id, "UpdatedFirst", "UpdatedLast"))
                .unwrap()
        });
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            DieselBench::update_user(&mut conn, id, "UpdatedFirst", "UpdatedLast").unwrap()
        });
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        let mut idx = 0;
        b.iter(|| {
            let id = user_ids[idx % user_ids.len()];
            idx += 1;
            rt.block_on(ClorindeBench::update_user(
                &client,
                id,
                "UpdatedFirst",
                "UpdatedLast",
            ))
            .unwrap()
        });
    });

    group.finish();
}

// ============================================================================
// Join Benchmarks
// ============================================================================

fn bench_join_posts_users(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("join_posts_users");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));

        let limit = *size as i64;

        // tokio-postgres
        group.bench_with_input(BenchmarkId::new("tokio_postgres", size), size, |b, _| {
            let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(TokioPostgresBench::select_posts_with_user(&client, limit))
                    .unwrap()
            });
        });

        // sqlx
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SqlxBench::select_posts_with_user(&pool, limit))
                    .unwrap()
            });
        });

        // sea-orm
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SeaOrmBench::select_posts_with_user(&db, *size as u64))
                    .unwrap()
            });
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            b.iter(|| DieselBench::select_posts_with_user(&mut conn, limit).unwrap());
        });

        // clorinde
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(ClorindeBench::select_posts_with_user(&client, limit))
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_join_triple(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("join_users_posts_comments");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);

    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));

        let limit = *size as i64;

        // tokio-postgres
        group.bench_with_input(BenchmarkId::new("tokio_postgres", size), size, |b, _| {
            let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(TokioPostgresBench::select_users_posts_comments(&client, limit))
                    .unwrap()
            });
        });

        // sqlx
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SqlxBench::select_users_posts_comments(&pool, limit))
                    .unwrap()
            });
        });

        // sea-orm (note: less efficient due to ORM limitations)
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(SeaOrmBench::select_users_posts_comments(&db, *size as u64))
                    .unwrap()
            });
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            b.iter(|| DieselBench::select_users_posts_comments(&mut conn, limit).unwrap());
        });

        // clorinde
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            b.iter(|| {
                rt.block_on(ClorindeBench::select_users_posts_comments(&client, limit))
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// Aggregate Benchmarks
// ============================================================================

fn bench_aggregate_count(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("aggregate_count_posts_per_user");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(TokioPostgresBench::count_posts_per_user(&client))
                .unwrap()
        });
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        b.iter(|| rt.block_on(SqlxBench::count_posts_per_user(&pool)).unwrap());
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        b.iter(|| rt.block_on(SeaOrmBench::count_posts_per_user(&db)).unwrap());
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        b.iter(|| DieselBench::count_posts_per_user(&mut conn).unwrap());
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(ClorindeBench::count_posts_per_user(&client))
                .unwrap()
        });
    });

    group.finish();
}

// ============================================================================
// Transaction Benchmarks
// ============================================================================

fn bench_transaction_insert(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("transaction_insert_user_with_posts");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);

    for size in &[1, 5, 10] {
        let posts: Vec<NewPost> = (0..*size)
            .map(|i| NewPost::generate(Uuid::nil(), i))
            .collect();

        // sqlx (has proper transaction support)
        group.bench_with_input(BenchmarkId::new("sqlx", size), size, |b, _| {
            let pool = rt.block_on(SqlxBench::connect()).unwrap();
            let mut counter = 0usize;
            b.iter(|| {
                counter += 1;
                let user = NewUser::generate(counter);
                rt.block_on(SqlxBench::insert_user_with_posts(&pool, &user, &posts))
                    .unwrap()
            });
            rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
        });

        // sea-orm
        group.bench_with_input(BenchmarkId::new("sea_orm", size), size, |b, _| {
            let db = rt.block_on(SeaOrmBench::connect()).unwrap();
            let mut counter = 0usize;
            b.iter(|| {
                counter += 1;
                let user = NewUser::generate(counter);
                rt.block_on(SeaOrmBench::insert_user_with_posts(&db, &user, &posts))
                    .unwrap()
            });
            rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
        });

        // diesel
        group.bench_with_input(BenchmarkId::new("diesel", size), size, |b, _| {
            let pool = DieselBench::connect().unwrap();
            let mut conn = pool.get().unwrap();
            let mut counter = 0usize;
            b.iter(|| {
                counter += 1;
                let user = NewUser::generate(counter);
                DieselBench::insert_user_with_posts(&mut conn, &user, &posts).unwrap()
            });
            DieselBench::cleanup(&mut conn).unwrap();
        });

        // clorinde (using sequential inserts)
        group.bench_with_input(BenchmarkId::new("clorinde", size), size, |b, _| {
            let client = rt.block_on(ClorindeBench::connect()).unwrap();
            let mut counter = 0usize;
            b.iter(|| {
                counter += 1;
                let user = NewUser::generate(counter);
                rt.block_on(ClorindeBench::insert_user_with_posts(&client, &user, &posts))
                    .unwrap()
            });
            rt.block_on(ClorindeBench::cleanup(&client)).unwrap();
        });
    }

    group.finish();
}

// ============================================================================
// Heavy Workload Benchmarks
// ============================================================================

fn bench_heavy_mixed_workload(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("heavy_mixed_workload");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20);

    // Heavy workload: mix of reads (80%) and writes (20%)
    let operations = 100;

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    counter += 1;
                    if i % 5 == 0 {
                        // Write (20%)
                        let user = NewUser::generate(counter);
                        let _ = TokioPostgresBench::insert_user(&client, &user).await;
                    } else {
                        // Read (80%)
                        let _ = TokioPostgresBench::select_users_limit(&client, 50).await;
                    }
                }
            });
        });
        rt.block_on(TokioPostgresBench::cleanup(&client)).unwrap();
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    counter += 1;
                    if i % 5 == 0 {
                        let user = NewUser::generate(counter);
                        let _ = SqlxBench::insert_user(&pool, &user).await;
                    } else {
                        let _ = SqlxBench::select_users_limit(&pool, 50).await;
                    }
                }
            });
        });
        rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    counter += 1;
                    if i % 5 == 0 {
                        let user = NewUser::generate(counter);
                        let _ = SeaOrmBench::insert_user(&db, &user).await;
                    } else {
                        let _ = SeaOrmBench::select_users_limit(&db, 50).await;
                    }
                }
            });
        });
        rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            for i in 0..operations {
                counter += 1;
                if i % 5 == 0 {
                    let user = NewUser::generate(counter);
                    let _ = DieselBench::insert_user(&mut conn, &user);
                } else {
                    let _ = DieselBench::select_users_limit(&mut conn, 50);
                }
            }
        });
        DieselBench::cleanup(&mut conn).unwrap();
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    counter += 1;
                    if i % 5 == 0 {
                        let user = NewUser::generate(counter);
                        let _ = ClorindeBench::insert_user(&client, &user).await;
                    } else {
                        let _ = ClorindeBench::select_users_limit(&client, 50).await;
                    }
                }
            });
        });
        rt.block_on(ClorindeBench::cleanup(&client)).unwrap();
    });

    group.finish();
}

fn bench_heavy_read_intensive(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("heavy_read_intensive");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(30);

    let operations = 200;

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    match i % 4 {
                        0 => {
                            let _ = TokioPostgresBench::select_users_limit(&client, 100).await;
                        }
                        1 => {
                            let _ =
                                TokioPostgresBench::select_users_filtered(&client, 25, 55, 50)
                                    .await;
                        }
                        2 => {
                            let _ = TokioPostgresBench::select_posts_with_user(&client, 50).await;
                        }
                        _ => {
                            let _ = TokioPostgresBench::count_posts_per_user(&client).await;
                        }
                    }
                }
            });
        });
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    match i % 4 {
                        0 => {
                            let _ = SqlxBench::select_users_limit(&pool, 100).await;
                        }
                        1 => {
                            let _ = SqlxBench::select_users_filtered(&pool, 25, 55, 50).await;
                        }
                        2 => {
                            let _ = SqlxBench::select_posts_with_user(&pool, 50).await;
                        }
                        _ => {
                            let _ = SqlxBench::count_posts_per_user(&pool).await;
                        }
                    }
                }
            });
        });
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    match i % 4 {
                        0 => {
                            let _ = SeaOrmBench::select_users_limit(&db, 100).await;
                        }
                        1 => {
                            let _ = SeaOrmBench::select_users_filtered(&db, 25, 55, 50).await;
                        }
                        2 => {
                            let _ = SeaOrmBench::select_posts_with_user(&db, 50).await;
                        }
                        _ => {
                            let _ = SeaOrmBench::count_posts_per_user(&db).await;
                        }
                    }
                }
            });
        });
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        b.iter(|| {
            for i in 0..operations {
                match i % 4 {
                    0 => {
                        let _ = DieselBench::select_users_limit(&mut conn, 100);
                    }
                    1 => {
                        let _ = DieselBench::select_users_filtered(&mut conn, 25, 55, 50);
                    }
                    2 => {
                        let _ = DieselBench::select_posts_with_user(&mut conn, 50);
                    }
                    _ => {
                        let _ = DieselBench::count_posts_per_user(&mut conn);
                    }
                }
            }
        });
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        b.iter(|| {
            rt.block_on(async {
                for i in 0..operations {
                    match i % 4 {
                        0 => {
                            let _ = ClorindeBench::select_users_limit(&client, 100).await;
                        }
                        1 => {
                            let _ =
                                ClorindeBench::select_users_filtered(&client, 25, 55, 50).await;
                        }
                        2 => {
                            let _ = ClorindeBench::select_posts_with_user(&client, 50).await;
                        }
                        _ => {
                            let _ = ClorindeBench::count_posts_per_user(&client).await;
                        }
                    }
                }
            });
        });
    });

    group.finish();
}

fn bench_heavy_write_intensive(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("heavy_write_intensive");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    let batch_size = 50;

    // tokio-postgres
    group.bench_function("tokio_postgres", |b| {
        let client = rt.block_on(TokioPostgresBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..batch_size {
                    counter += 1;
                    let user = NewUser::generate(counter);
                    let user_id = TokioPostgresBench::insert_user(&client, &user).await.unwrap();
                    
                    // Insert a post for this user
                    let post = NewPost::generate(user_id, counter);
                    TokioPostgresBench::insert_post(&client, &post).await.unwrap();
                    
                    // Update the user
                    TokioPostgresBench::update_user(&client, user_id, "Modified", "Name")
                        .await
                        .unwrap();
                }
            });
        });
        rt.block_on(TokioPostgresBench::cleanup(&client)).unwrap();
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..batch_size {
                    counter += 1;
                    let user = NewUser::generate(counter);
                    let user_id = SqlxBench::insert_user(&pool, &user).await.unwrap();
                    
                    let post = NewPost::generate(user_id, counter);
                    SqlxBench::insert_post(&pool, &post).await.unwrap();
                    
                    SqlxBench::update_user(&pool, user_id, "Modified", "Name")
                        .await
                        .unwrap();
                }
            });
        });
        rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..batch_size {
                    counter += 1;
                    let user = NewUser::generate(counter);
                    let user_id = SeaOrmBench::insert_user(&db, &user).await.unwrap();
                    
                    let post = NewPost::generate(user_id, counter);
                    SeaOrmBench::insert_post(&db, &post).await.unwrap();
                    
                    SeaOrmBench::update_user(&db, user_id, "Modified", "Name")
                        .await
                        .unwrap();
                }
            });
        });
        rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
    });

    // diesel
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect().unwrap();
        let mut conn = pool.get().unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            for _ in 0..batch_size {
                counter += 1;
                let user = NewUser::generate(counter);
                let user_id = DieselBench::insert_user(&mut conn, &user).unwrap();
                
                let post = NewPost::generate(user_id, counter);
                DieselBench::insert_post(&mut conn, &post).unwrap();
                
                DieselBench::update_user(&mut conn, user_id, "Modified", "Name").unwrap();
            }
        });
        DieselBench::cleanup(&mut conn).unwrap();
    });

    // clorinde
    group.bench_function("clorinde", |b| {
        let client = rt.block_on(ClorindeBench::connect()).unwrap();
        let mut counter = 0usize;
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..batch_size {
                    counter += 1;
                    let user = NewUser::generate(counter);
                    let user_id = ClorindeBench::insert_user(&client, &user).await.unwrap();
                    
                    let post = NewPost::generate(user_id, counter);
                    ClorindeBench::insert_post(&client, &post).await.unwrap();
                    
                    ClorindeBench::update_user(&client, user_id, "Modified", "Name")
                        .await
                        .unwrap();
                }
            });
        });
        rt.block_on(ClorindeBench::cleanup(&client)).unwrap();
    });

    group.finish();
}

// ============================================================================
// Concurrent Query Benchmarks (Connection Pooling)
// ============================================================================

fn bench_concurrent_reads(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("concurrent_reads");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    // Test with different concurrency levels
    for concurrency in &[10, 50, 100] {
        group.throughput(Throughput::Elements(*concurrency as u64));

        // tokio-postgres with deadpool
        group.bench_with_input(
            BenchmarkId::new("tokio_postgres_pooled", concurrency),
            concurrency,
            |b, &conc| {
                let pool = TokioPostgresBench::create_pool(conc);
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = Vec::with_capacity(conc);
                        for _ in 0..conc {
                            let pool = pool.clone();
                            handles.push(tokio::spawn(async move {
                                TokioPostgresBench::pooled_select_users_limit(&pool, 50).await
                            }));
                        }
                        for handle in handles {
                            let _ = handle.await;
                        }
                    });
                });
            },
        );

        // sqlx (already pooled)
        group.bench_with_input(BenchmarkId::new("sqlx", concurrency), concurrency, |b, &conc| {
            let pool = rt.block_on(SqlxBench::connect_with_pool_size(conc as u32)).unwrap();
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = Vec::with_capacity(conc);
                    for _ in 0..conc {
                        let pool = pool.clone();
                        handles.push(tokio::spawn(async move {
                            SqlxBench::select_users_limit(&pool, 50).await
                        }));
                    }
                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            });
        });

        // sea-orm (uses sqlx pool)
        group.bench_with_input(BenchmarkId::new("sea_orm", concurrency), concurrency, |b, &conc| {
            let db = rt.block_on(SeaOrmBench::connect_with_pool_size(conc as u32)).unwrap();
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = Vec::with_capacity(conc);
                    for _ in 0..conc {
                        let db = db.clone();
                        handles.push(tokio::spawn(async move {
                            SeaOrmBench::select_users_limit(&db, 50).await
                        }));
                    }
                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            });
        });

        // diesel with r2d2 (sync - uses thread pool)
        group.bench_with_input(BenchmarkId::new("diesel", concurrency), concurrency, |b, &conc| {
            let pool = DieselBench::connect_with_pool_size(conc as u32).unwrap();
            b.iter(|| {
                let pool = pool.clone();
                std::thread::scope(|s| {
                    for _ in 0..conc {
                        let pool = pool.clone();
                        s.spawn(move || {
                            let mut conn = pool.get().unwrap();
                            let _ = DieselBench::select_users_limit(&mut conn, 50);
                        });
                    }
                });
            });
        });
    }

    group.finish();
}

fn bench_concurrent_mixed(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("concurrent_mixed_workload");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(15);

    let concurrency = 50;
    let ops_per_task = 20;

    // tokio-postgres with deadpool
    group.bench_function("tokio_postgres_pooled", |b| {
        let pool = TokioPostgresBench::create_pool(concurrency);
        let counter = std::sync::atomic::AtomicUsize::new(0);
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(concurrency);
                for _ in 0..concurrency {
                    let pool = pool.clone();
                    let cnt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    handles.push(tokio::spawn(async move {
                        for i in 0..ops_per_task {
                            if (cnt + i) % 5 == 0 {
                                let user = NewUser::generate(cnt * 1000 + i);
                                let _ = TokioPostgresBench::pooled_insert_user(&pool, &user).await;
                            } else {
                                let _ = TokioPostgresBench::pooled_select_users_limit(&pool, 50).await;
                            }
                        }
                    }));
                }
                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
        rt.block_on(TokioPostgresBench::pooled_cleanup(&pool)).unwrap();
    });

    // sqlx
    group.bench_function("sqlx", |b| {
        let pool = rt.block_on(SqlxBench::connect_with_pool_size(concurrency as u32)).unwrap();
        let counter = std::sync::atomic::AtomicUsize::new(0);
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(concurrency);
                for _ in 0..concurrency {
                    let pool = pool.clone();
                    let cnt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    handles.push(tokio::spawn(async move {
                        for i in 0..ops_per_task {
                            if (cnt + i) % 5 == 0 {
                                let user = NewUser::generate(cnt * 1000 + i);
                                let _ = SqlxBench::insert_user(&pool, &user).await;
                            } else {
                                let _ = SqlxBench::select_users_limit(&pool, 50).await;
                            }
                        }
                    }));
                }
                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
        rt.block_on(SqlxBench::cleanup(&pool)).unwrap();
    });

    // sea-orm
    group.bench_function("sea_orm", |b| {
        let db = rt.block_on(SeaOrmBench::connect_with_pool_size(concurrency as u32)).unwrap();
        let counter = std::sync::atomic::AtomicUsize::new(0);
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(concurrency);
                for _ in 0..concurrency {
                    let db = db.clone();
                    let cnt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    handles.push(tokio::spawn(async move {
                        for i in 0..ops_per_task {
                            if (cnt + i) % 5 == 0 {
                                let user = NewUser::generate(cnt * 1000 + i);
                                let _ = SeaOrmBench::insert_user(&db, &user).await;
                            } else {
                                let _ = SeaOrmBench::select_users_limit(&db, 50).await;
                            }
                        }
                    }));
                }
                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
        rt.block_on(SeaOrmBench::cleanup(&db)).unwrap();
    });

    // diesel with r2d2
    group.bench_function("diesel", |b| {
        let pool = DieselBench::connect_with_pool_size(concurrency as u32).unwrap();
        let counter = std::sync::atomic::AtomicUsize::new(0);
        b.iter(|| {
            let pool = pool.clone();
            std::thread::scope(|s| {
                for _ in 0..concurrency {
                    let pool = pool.clone();
                    let cnt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    s.spawn(move || {
                        let mut conn = pool.get().unwrap();
                        for i in 0..ops_per_task {
                            if (cnt + i) % 5 == 0 {
                                let user = NewUser::generate(cnt * 1000 + i);
                                let _ = DieselBench::insert_user(&mut conn, &user);
                            } else {
                                let _ = DieselBench::select_users_limit(&mut conn, 50);
                            }
                        }
                    });
                }
            });
        });
        let mut conn = pool.get().unwrap();
        DieselBench::cleanup(&mut conn).unwrap();
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    // Insert benchmarks
    bench_insert_single,
    bench_insert_batch,
    // Select benchmarks
    bench_select_by_id,
    bench_select_limit,
    bench_select_filtered,
    // Update benchmarks
    bench_update_user,
    // Join benchmarks
    bench_join_posts_users,
    bench_join_triple,
    // Aggregate benchmarks
    bench_aggregate_count,
    // Transaction benchmarks
    bench_transaction_insert,
    // Heavy workload benchmarks
    bench_heavy_mixed_workload,
    bench_heavy_read_intensive,
    bench_heavy_write_intensive,
    // Concurrent benchmarks
    bench_concurrent_reads,
    bench_concurrent_mixed,
);

criterion_main!(benches);
