#!/bin/bash

# PostgreSQL Library Benchmark Runner
# Comprehensive benchmark suite for comparing:
# - tokio-postgres
# - sqlx
# - sea-orm  
# - diesel
# - clorinde

set -e

echo "==================================================================="
echo "  PostgreSQL Library Benchmark Suite"
echo "  Comparing: tokio-postgres, sqlx, sea-orm, diesel, clorinde"
echo "==================================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${CYAN}=== $1 ===${NC}"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    print_success "Docker is running"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed."
        exit 1
    fi
    
    # Check for docker compose (v2) or docker-compose (v1)
    if docker compose version &> /dev/null; then
        DOCKER_COMPOSE="docker compose"
    elif docker-compose --version &> /dev/null; then
        DOCKER_COMPOSE="docker-compose"
    else
        print_error "Docker Compose is not installed."
        exit 1
    fi
    
    print_success "All dependencies are available"
    print_status "Using: $DOCKER_COMPOSE"
}

# Start PostgreSQL database
start_database() {
    print_header "Starting PostgreSQL Database"
    
    # Stop any existing container
    $DOCKER_COMPOSE down -v 2>/dev/null || true
    
    # Remove old volume to ensure clean state
    docker volume rm playground_postgres_data 2>/dev/null || true
    
    # Start database
    $DOCKER_COMPOSE up -d
    
    # Wait for database to be ready
    print_status "Waiting for database to be ready..."
    for i in {1..60}; do
        if $DOCKER_COMPOSE exec -T postgres pg_isready -U benchmark_user -d benchmark_db > /dev/null 2>&1; then
            print_success "Database is ready!"
            return 0
        fi
        echo -n "."
        sleep 1
    done
    
    print_error "Database failed to start within 60 seconds"
    $DOCKER_COMPOSE logs postgres
    exit 1
}

# Verify database schema and data
verify_database() {
    print_header "Verifying Database"
    
    # Check if tables exist
    tables=$($DOCKER_COMPOSE exec -T postgres psql -U benchmark_user -d benchmark_db -t -c \
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';" 2>/dev/null | tr -d ' ')
    
    if [ "$tables" -ge "5" ]; then
        print_success "Database schema is ready ($tables tables found)"
    else
        print_warning "Expected at least 5 tables, found $tables"
    fi
    
    # Show data counts
    print_status "Data statistics:"
    $DOCKER_COMPOSE exec -T postgres psql -U benchmark_user -d benchmark_db -c \
        "SELECT 'users' as table_name, COUNT(*) as row_count FROM users
         UNION ALL
         SELECT 'posts', COUNT(*) FROM posts
         UNION ALL
         SELECT 'comments', COUNT(*) FROM comments
         UNION ALL
         SELECT 'tags', COUNT(*) FROM tags
         UNION ALL
         SELECT 'post_tags', COUNT(*) FROM post_tags
         ORDER BY table_name;" 2>/dev/null || true
}

# Build benchmark project
build_benchmarks() {
    print_header "Building Benchmark Project"
    
    # Set environment variables
    export DATABASE_URL="postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
    
    # Build in release mode
    if cargo build --release; then
        print_success "Benchmark project built successfully"
    else
        print_error "Failed to build benchmark project"
        exit 1
    fi
}

# Run quick smoke test
run_smoke_test() {
    print_header "Running Smoke Test"
    
    export DATABASE_URL="postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
    
    if timeout 120 cargo bench -- insert_single --sample-size 10 --warm-up-time 1 --measurement-time 3 2>&1 | head -100; then
        print_success "Smoke test passed"
    else
        print_warning "Smoke test had issues (this might be OK)"
    fi
}

# Run full benchmark suite
run_full_benchmarks() {
    print_header "Running Full Benchmark Suite"
    print_warning "This may take 30-60 minutes depending on your hardware"
    
    export DATABASE_URL="postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
    
    # Create results directory with timestamp
    RESULTS_DIR="benchmark-results-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$RESULTS_DIR"
    
    print_status "Results will be saved to: $RESULTS_DIR"
    
    # Run benchmarks and save results
    if cargo bench 2>&1 | tee "$RESULTS_DIR/benchmark.log"; then
        print_success "Benchmarks completed successfully"
        
        # Copy criterion results
        if [ -d "target/criterion" ]; then
            cp -r target/criterion "$RESULTS_DIR/"
            print_success "Results copied to $RESULTS_DIR/criterion/"
        fi
        
        # Generate summary
        generate_summary "$RESULTS_DIR"
        
    else
        print_error "Benchmarks failed. Check $RESULTS_DIR/benchmark.log for details."
        exit 1
    fi
}

# Run specific benchmark group
run_benchmark_group() {
    local group="$1"
    print_header "Running Benchmark Group: $group"
    
    export DATABASE_URL="postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
    
    cargo bench -- "$group"
}

# Generate benchmark summary
generate_summary() {
    local results_dir="$1"
    
    print_status "Generating benchmark summary..."
    
    cat > "$results_dir/SUMMARY.md" << 'EOF'
# PostgreSQL Library Benchmark Results

## Overview

This benchmark compares five popular Rust PostgreSQL libraries:

| Library | Type | Async | Features |
|---------|------|-------|----------|
| tokio-postgres | Low-level driver | Yes | Direct database access, minimal overhead |
| sqlx | SQL toolkit | Yes | Compile-time checked queries, connection pooling |
| sea-orm | ORM | Yes | Active record pattern, migrations |
| diesel | ORM | No (sync) | Type-safe query builder, schema DSL |
| clorinde | Code generator | Yes | Generated type-safe queries from SQL |

## Benchmark Categories

### 1. Insert Operations
- **insert_single_user**: Single row insert performance
- **insert_batch_users**: Batch insert with varying sizes (10, 100, 1000)

### 2. Select Operations
- **select_user_by_id**: Primary key lookup performance
- **select_users_limit**: Simple SELECT with LIMIT
- **select_users_filtered**: SELECT with WHERE clause and multiple conditions

### 3. Update Operations
- **update_user**: Single row update by primary key

### 4. Join Operations
- **join_posts_users**: Two-table JOIN performance
- **join_users_posts_comments**: Three-table JOIN performance

### 5. Aggregate Operations
- **aggregate_count_posts_per_user**: GROUP BY with COUNT aggregation

### 6. Transaction Operations
- **transaction_insert_user_with_posts**: Multi-statement transaction

### 7. Heavy Workload Simulation
- **heavy_mixed_workload**: 80% reads, 20% writes simulation
- **heavy_read_intensive**: Read-heavy workload (200 operations)
- **heavy_write_intensive**: Write-heavy workload with updates

## Key Findings

[Results will be populated after running the benchmarks]

## How to Read the Results

- Lower times are better
- Results are in microseconds (μs) or milliseconds (ms)
- Each benchmark runs multiple iterations for statistical accuracy
- Check the `criterion/` directory for detailed HTML reports

## Hardware/Software Information

EOF

    # Add system info
    echo '```' >> "$results_dir/SUMMARY.md"
    echo "Date: $(date)" >> "$results_dir/SUMMARY.md"
    echo "OS: $(uname -s) $(uname -r)" >> "$results_dir/SUMMARY.md"
    echo "CPU: $(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d':' -f2 | sed 's/^ *//' || sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')" >> "$results_dir/SUMMARY.md"
    echo "Memory: $(free -h 2>/dev/null | grep '^Mem:' | awk '{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 "G"}' || echo 'Unknown')" >> "$results_dir/SUMMARY.md"
    echo "Rust: $(rustc --version)" >> "$results_dir/SUMMARY.md"
    echo "Cargo: $(cargo --version)" >> "$results_dir/SUMMARY.md"
    echo "Docker: $(docker --version)" >> "$results_dir/SUMMARY.md"
    echo '```' >> "$results_dir/SUMMARY.md"
    
    print_success "Summary generated: $results_dir/SUMMARY.md"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    if [ "$KEEP_DB" != "true" ]; then
        $DOCKER_COMPOSE down -v 2>/dev/null || true
        print_success "Database stopped and volumes removed"
    else
        print_status "Database kept running (KEEP_DB=true)"
        print_status "Database URL: postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
    fi
}

# Show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  full        Run full benchmark suite (default)"
    echo "  quick       Run quick smoke test only"
    echo "  group NAME  Run specific benchmark group"
    echo "  setup       Set up database only (don't run benchmarks)"
    echo ""
    echo "Options:"
    echo "  --keep-db   Keep database running after benchmarks"
    echo "  --no-build  Skip building (use existing binary)"
    echo "  --help      Show this help message"
    echo ""
    echo "Available benchmark groups:"
    echo "  insert      Insert benchmarks"
    echo "  select      Select benchmarks"
    echo "  update      Update benchmarks"
    echo "  join        Join benchmarks"
    echo "  aggregate   Aggregate benchmarks"
    echo "  transaction Transaction benchmarks"
    echo "  heavy       Heavy workload benchmarks"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run full benchmarks"
    echo "  $0 quick              # Run quick smoke test"
    echo "  $0 group insert       # Run only insert benchmarks"
    echo "  $0 --keep-db full     # Run full benchmarks, keep DB running"
}

# Main execution
main() {
    cd "$(dirname "$0")"
    
    # Parse command line arguments
    KEEP_DB=false
    NO_BUILD=false
    COMMAND="full"
    GROUP=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --keep-db)
                KEEP_DB=true
                shift
                ;;
            --no-build)
                NO_BUILD=true
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            full|quick|setup)
                COMMAND="$1"
                shift
                ;;
            group)
                COMMAND="group"
                GROUP="$2"
                shift 2
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Trap cleanup on exit
    trap cleanup EXIT
    
    # Run benchmark pipeline
    check_docker
    check_dependencies
    start_database
    
    # Wait a bit for data to be fully loaded
    sleep 5
    verify_database
    
    if [ "$COMMAND" = "setup" ]; then
        print_success "Database setup complete!"
        KEEP_DB=true
        exit 0
    fi
    
    if [ "$NO_BUILD" != "true" ]; then
        build_benchmarks
    fi
    
    case $COMMAND in
        quick)
            run_smoke_test
            print_success "Quick smoke test completed!"
            ;;
        group)
            if [ -z "$GROUP" ]; then
                print_error "No benchmark group specified"
                exit 1
            fi
            run_benchmark_group "$GROUP"
            ;;
        full)
            run_full_benchmarks
            print_success "Full benchmark suite completed!"
            print_status "Check results directory for detailed reports."
            ;;
    esac
    
    # Show database status if keeping it
    if [ "$KEEP_DB" = "true" ]; then
        print_status "Database is still running."
        print_status "Connection: postgres://benchmark_user:benchmark_pass@localhost:5432/benchmark_db"
        print_status "To stop: docker compose down -v"
    fi
}

# Run main function
main "$@"
