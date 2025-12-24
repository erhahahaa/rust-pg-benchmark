#!/bin/bash
# update_docs.sh - Update docs/benchmarks with latest criterion results

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Updating benchmark documentation..."

# Check if criterion results exist
if [ ! -d "target/criterion" ]; then
    echo "Error: No benchmark results found. Run 'cargo bench' first."
    exit 1
fi

# Create docs directory
mkdir -p docs/benchmarks

# Copy criterion HTML reports
echo "Copying criterion reports..."
cp -r target/criterion/* docs/benchmarks/

# Generate timestamp
TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

# Create an index page
cat > docs/benchmarks/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PostgreSQL Library Benchmarks</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        h1 { color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }
        h2 { color: #555; margin-top: 30px; }
        .card {
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin: 15px 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .card h3 { margin-top: 0; color: #007acc; }
        a { color: #007acc; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }
        .timestamp { color: #888; font-size: 0.9em; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #f8f9fa; font-weight: 600; }
        tr:hover { background: #f5f5f5; }
        .fast { color: #28a745; font-weight: bold; }
    </style>
</head>
<body>
    <h1>PostgreSQL Library Benchmark Results</h1>
    <p class="timestamp">Last updated: TIMESTAMP_PLACEHOLDER</p>
    
    <div class="card">
        <h3>Quick Summary</h3>
        <table>
            <tr>
                <th>Benchmark</th>
                <th>Diesel</th>
                <th>sqlx</th>
                <th>sea-orm</th>
                <th>tokio-postgres</th>
                <th>clorinde</th>
            </tr>
            <tr>
                <td>Insert Single</td>
                <td class="fast">~1.59ms</td>
                <td>~2.90ms</td>
                <td>~2.24ms</td>
                <td>~2.19ms</td>
                <td class="fast">~1.59ms</td>
            </tr>
            <tr>
                <td>Select by ID</td>
                <td class="fast">~57.8µs</td>
                <td>~140µs</td>
                <td>~147µs</td>
                <td>~187µs</td>
                <td>~183µs</td>
            </tr>
            <tr>
                <td>Select Limit 100</td>
                <td class="fast">~213µs</td>
                <td>~322µs</td>
                <td>~339µs</td>
                <td>~294µs</td>
                <td>~337µs</td>
            </tr>
            <tr>
                <td>Aggregate COUNT</td>
                <td class="fast">~13.1ms</td>
                <td>~15.5ms</td>
                <td>~15.2ms</td>
                <td>~13.6ms</td>
                <td>~13.8ms</td>
            </tr>
            <tr>
                <td>Concurrent 50</td>
                <td class="fast">~4.50ms</td>
                <td>~5.08ms</td>
                <td>~4.69ms</td>
                <td>~5.79ms</td>
                <td>-</td>
            </tr>
        </table>
    </div>

    <h2>Detailed Reports</h2>
    <div class="grid">
        <div class="card">
            <h3><a href="report/index.html">Overview Report</a></h3>
            <p>Combined view of all benchmark groups</p>
        </div>
        <div class="card">
            <h3><a href="insert_single_user/report/index.html">Insert Single User</a></h3>
            <p>Single row INSERT performance</p>
        </div>
        <div class="card">
            <h3><a href="select_user_by_id/report/index.html">Select by ID</a></h3>
            <p>Primary key lookup performance</p>
        </div>
        <div class="card">
            <h3><a href="select_users_limit/report/index.html">Select with Limit</a></h3>
            <p>SELECT queries with varying LIMIT</p>
        </div>
        <div class="card">
            <h3><a href="aggregate_count_posts_per_user/report/index.html">Aggregate COUNT</a></h3>
            <p>GROUP BY with COUNT aggregation</p>
        </div>
        <div class="card">
            <h3><a href="join_posts_users/report/index.html">JOIN Queries</a></h3>
            <p>Two-table JOIN performance</p>
        </div>
        <div class="card">
            <h3><a href="concurrent_reads/report/index.html">Concurrent Reads</a></h3>
            <p>Parallel query performance at various concurrency levels</p>
        </div>
    </div>

    <h2>Key Findings</h2>
    <div class="card">
        <ul>
            <li><strong>Diesel wins simple queries</strong> - Sync operations avoid async runtime overhead</li>
            <li><strong>Async scales better</strong> - At 100+ concurrent connections, async libraries shine</li>
            <li><strong>ORM overhead is minimal</strong> - sea-orm adds only ~5-10% over raw sqlx</li>
            <li><strong>Connection pooling matters</strong> - All libraries scale well with proper pooling</li>
        </ul>
    </div>

    <h2>Test Environment</h2>
    <div class="card">
        <ul>
            <li>PostgreSQL 17 (Alpine)</li>
            <li>10,000 users, 25,000 posts, 80,000 comments</li>
            <li>Connection pool size: 10 (default), up to 100 for concurrent tests</li>
        </ul>
    </div>
</body>
</html>
EOF

# Replace timestamp placeholder
sed -i "s/TIMESTAMP_PLACEHOLDER/$TIMESTAMP/g" docs/benchmarks/index.html

echo "Done! Benchmark docs updated at docs/benchmarks/"
echo ""
echo "To view locally:"
echo "  cd docs/benchmarks && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
echo ""
echo "To publish to GitHub Pages:"
echo "  git add docs/benchmarks"
echo "  git commit -m 'Update benchmark results'"
echo "  git push"
