# PostgreSQL Library Benchmark Suite

Comprehensive and fair benchmark comparison of popular Rust PostgreSQL libraries.

[![Benchmark Results](https://img.shields.io/badge/benchmarks-view%20results-blue)](https://yourusername.github.io/pg-benchmark/benchmarks/report/)

## Benchmark Results

> **[View Interactive Charts](docs/benchmarks/report/index.html)** | [Full Report](docs/benchmarks/)

### Quick Summary

| Benchmark | Diesel | sqlx | sea-orm | tokio-postgres | clorinde |
|-----------|--------|------|---------|----------------|----------|
| Insert Single | **1.59ms** | 2.90ms | 2.24ms | 2.19ms | 1.59ms |
| Select by ID | **57.8µs** | 140µs | 147µs | 187µs | 183µs |
| Select Limit 10 | **88µs** | 161µs | 169µs | 188µs | 182µs |
| Select Limit 100 | **213µs** | 322µs | 339µs | 294µs | 337µs |
| Aggregate COUNT | **13.1ms** | 15.5ms | 15.2ms | 13.6ms | 13.8ms |
| Concurrent 50 | **4.50ms** | 5.08ms | 4.69ms | 5.79ms | - |
| Concurrent 100 | 12.9ms | 11.9ms | **9.8ms** | 16.1ms | - |

*Lower is better. Bold indicates fastest.*

### Performance Charts

#### Select User by ID (Point Lookup)
```
diesel          ████████████████████                                      57.8µs
sqlx            ████████████████████████████████████████████████         140µs
sea-orm         █████████████████████████████████████████████████        147µs
clorinde        ████████████████████████████████████████████████████████ 183µs  
tokio-postgres  █████████████████████████████████████████████████████████ 187µs
```

#### Select Users with Limit (100 rows)
```
diesel          ████████████████████████████                              213µs
tokio-postgres  ██████████████████████████████████████                    294µs
sqlx            ██████████████████████████████████████████                322µs
clorinde        ████████████████████████████████████████████              337µs
sea-orm         ████████████████████████████████████████████              339µs
```

#### Concurrent Reads (50 parallel connections)
```
diesel          ████████████████████████████████████████████████          4.50ms
sea-orm         ██████████████████████████████████████████████████        4.69ms
sqlx            █████████████████████████████████████████████████████     5.08ms
tokio-postgres  ████████████████████████████████████████████████████████  5.79ms
```

#### Aggregate Query (COUNT posts per user)
```
diesel          ████████████████████████████████████████████████████      13.1ms
tokio-postgres  █████████████████████████████████████████████████████     13.6ms
clorinde        ██████████████████████████████████████████████████████    13.8ms
sea-orm         ████████████████████████████████████████████████████████  15.2ms
sqlx            █████████████████████████████████████████████████████████ 15.5ms
```

### Key Findings

| Observation | Details |
|-------------|---------|
| **Diesel wins simple queries** | Sync operations avoid async runtime overhead, 2-3x faster for point lookups |
| **Async scales better** | At 100+ concurrent connections, sea-orm/sqlx outperform diesel |
| **ORM overhead is minimal** | sea-orm adds only ~5-10% over raw sqlx |
| **Query complexity matters** | For complex aggregates, differences narrow as DB execution dominates |

### Recommendations

| Use Case | Recommended |
|----------|-------------|
| High-throughput sync service | **Diesel** |
| Async web server | **sqlx** or **sea-orm** |
| Rapid prototyping with ORM | **sea-orm** |
| Maximum control | **tokio-postgres** |
| Type-safe SQL from files | **clorinde** |

---

## Libraries Compared

| Library | Version | Type | Description |
|---------|---------|------|-------------|
| **tokio-postgres** | 0.7.x | Async Driver | Low-level async PostgreSQL driver |
| **sqlx** | 0.8.x | Async Toolkit | Compile-time checked SQL queries |
| **sea-orm** | 1.1.x | Async ORM | Active Record pattern ORM |
| **diesel** | 2.2.x | Sync ORM | Type-safe query builder |
| **clorinde** | 1.2.x | Code Generator | Generated type-safe queries from SQL |

## Quick Start

```bash
# Clone the repo
git clone https://github.com/yourusername/pg-benchmark.git
cd pg-benchmark

# Make the script executable
chmod +x run_benchmarks.sh

# Run the full benchmark suite
./run_benchmarks.sh

# Or run specific benchmark groups
./run_benchmarks.sh group insert
./run_benchmarks.sh group select
./run_benchmarks.sh group concurrent
```

## Prerequisites

- **Rust** 1.75+ (for async trait support)
- **Docker** with Docker Compose
- **4GB+ RAM** recommended for heavy workload tests

## Benchmark Categories

### 1. Insert Operations
- Single row inserts
- Batch inserts (10, 100, 1000 rows)

### 2. Select Operations  
- Primary key lookups (`select_user_by_id`)
- Simple SELECT with LIMIT (`select_users_limit`)
- Filtered SELECT with WHERE clauses (`select_users_filtered`)

### 3. Update Operations
- Single row updates by primary key

### 4. Join Operations
- Two-table JOINs (posts + users)
- Three-table JOINs (users + posts + comments)

### 5. Aggregate Operations
- GROUP BY with COUNT (`aggregate_count_posts_per_user`)

### 6. Transaction Operations
- Multi-statement transactions

### 7. Concurrent Operations (NEW)
- **concurrent_reads**: 10/50/100 parallel SELECT queries
- **concurrent_mixed_workload**: 50 connections with 80% reads, 20% writes

### 8. Heavy Workload Simulation
- **Mixed Workload**: 80% reads, 20% writes (100 operations)
- **Read Intensive**: 200 sequential read operations
- **Write Intensive**: Bulk inserts with updates (50 operations)

## Database Configuration

The benchmark uses PostgreSQL 17 with optimized settings:

```yaml
# Key PostgreSQL settings for heavy workload benchmarking
max_connections: 300
shared_buffers: 512MB
effective_cache_size: 2GB
work_mem: 16MB
max_parallel_workers: 8
max_parallel_workers_per_gather: 4
```

## Sample Data

The database is initialized with:
- **10,000 users** with realistic names and ages
- **25,000 posts** with varied content and statuses
- **80,000 comments** distributed across posts
- **100 tags** with random colors
- **Post-tag relationships** for many-to-many testing

## Running Benchmarks

### Full Suite
```bash
./run_benchmarks.sh full
```

### Quick Test
```bash
./run_benchmarks.sh quick
```

### Specific Benchmarks
```bash
# By group
./run_benchmarks.sh group insert
./run_benchmarks.sh group select
./run_benchmarks.sh group concurrent

# By name pattern
cargo bench -- select_user_by_id
cargo bench -- concurrent_reads
cargo bench -- aggregate
```

### Manual Benchmark Run
```bash
# Start database
docker compose up -d

# Wait for initialization (may take 30+ seconds for data generation)
sleep 30

# Run benchmarks
DATABASE_URL="postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db" \
  cargo bench

# Stop database
docker compose down -v
```

## Viewing Results

### Local HTML Reports
After running benchmarks, open in browser:
```bash
# Main report
open target/criterion/report/index.html

# Or serve locally
cd target/criterion && python3 -m http.server 8080
# Then visit http://localhost:8080
```

### GitHub Pages
Results are published to: `https://yourusername.github.io/pg-benchmark/benchmarks/report/`

## Fairness Considerations

This benchmark aims to be fair by:

1. **Same SQL**: All libraries execute equivalent SQL queries
2. **Same Connection Setup**: Similar connection pooling where applicable
3. **Same Data**: All tests use the same database state
4. **Warm-up**: Criterion handles warm-up automatically
5. **Multiple Iterations**: Statistical significance through repetition
6. **Cleanup**: Benchmark data is cleaned between runs

### Connection Pooling

| Library | Pool Type | Default Size |
|---------|-----------|--------------|
| tokio-postgres | deadpool-postgres | 10-100 (configurable) |
| sqlx | Built-in PgPool | 10 |
| sea-orm | Via SQLx | 10 |
| diesel | r2d2 | 10 |
| clorinde | deadpool-postgres | 10-100 (configurable) |

### Why Diesel Wins Simple Queries

Diesel (sync) often outperforms async libraries because:
1. **No async overhead** - No Future state machines, no runtime polling
2. **Direct function calls** - Simple call stack vs async context switching
3. **Optimized codegen** - Highly optimized query generation

Async advantages appear at **high concurrency** (100+ connections) where I/O multiplexing matters.

## Project Structure

```
.
├── Cargo.toml              # Project dependencies
├── compose.yml             # Docker Compose for PostgreSQL
├── init.sql                # Database schema and sample data
├── run_benchmarks.sh       # Benchmark runner script
├── README.md               # This file
├── docs/
│   └── benchmarks/         # Criterion HTML reports (for GitHub Pages)
├── src/
│   ├── lib.rs              # Shared types and traits
│   ├── main.rs             # Utility binary
│   ├── bench_tokio_postgres.rs
│   ├── bench_sqlx.rs
│   ├── bench_seaorm.rs
│   ├── bench_diesel.rs
│   └── bench_clorinde.rs
├── benches/
│   └── database_bench.rs   # Criterion benchmarks
└── clorinde_queries/       # Simulated Clorinde generated code
    ├── Cargo.toml
    └── src/lib.rs
```

## Troubleshooting

### Database Connection Failed
```bash
# Check if PostgreSQL is running
docker compose ps

# View logs
docker compose logs postgres

# Restart fresh
docker compose down -v && docker compose up -d
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Benchmark Timeout
Some benchmarks may take a while. The heavy workload tests run 20-30 seconds each.

## Test Environment

```
OS: Linux 6.17.9-arch1-1
CPU: 11th Gen Intel Core i5-11400H @ 2.70GHz
Memory: 16GB
Rust: 1.92.0
PostgreSQL: 17 (Alpine)
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add your benchmark scenarios
4. Run benchmarks and update results
5. Submit a pull request

## License

MIT License
# rust-pg-benchmark
