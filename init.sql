-- Initialize benchmark database schema for heavy workload testing
-- This schema is designed for comprehensive PostgreSQL library benchmarks

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For text search optimization

-- ============================================================================
-- Core Tables
-- ============================================================================

-- Users table for basic CRUD operations
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    age INTEGER CHECK (age >= 0 AND age <= 150),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Posts table for more complex operations
CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
    view_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Comments table for join operations
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tags table for many-to-many relationships
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) UNIQUE NOT NULL,
    color VARCHAR(7) DEFAULT '#000000',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Post tags junction table
CREATE TABLE post_tags (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

-- ============================================================================
-- Indexes for Performance
-- ============================================================================

-- Users indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at DESC);
CREATE INDEX idx_users_age ON users(age);
CREATE INDEX idx_users_first_name_trgm ON users USING gin(first_name gin_trgm_ops);
CREATE INDEX idx_users_last_name_trgm ON users USING gin(last_name gin_trgm_ops);

-- Posts indexes
CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_status ON posts(status);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX idx_posts_view_count ON posts(view_count DESC);
CREATE INDEX idx_posts_user_status ON posts(user_id, status);

-- Comments indexes
CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_comments_user_id ON comments(user_id);
CREATE INDEX idx_comments_created_at ON comments(created_at DESC);

-- Tags indexes
CREATE INDEX idx_tags_name ON tags(name);

-- ============================================================================
-- Sample Data Generation for Heavy Workload Testing
-- ============================================================================

-- Insert 10,000 users for realistic heavy load testing
INSERT INTO users (username, email, first_name, last_name, age)
SELECT 
    'user_' || i,
    'user_' || i || '@example.com',
    CASE (i % 20)
        WHEN 0 THEN 'James'
        WHEN 1 THEN 'Mary'
        WHEN 2 THEN 'John'
        WHEN 3 THEN 'Patricia'
        WHEN 4 THEN 'Robert'
        WHEN 5 THEN 'Jennifer'
        WHEN 6 THEN 'Michael'
        WHEN 7 THEN 'Linda'
        WHEN 8 THEN 'William'
        WHEN 9 THEN 'Elizabeth'
        WHEN 10 THEN 'David'
        WHEN 11 THEN 'Barbara'
        WHEN 12 THEN 'Richard'
        WHEN 13 THEN 'Susan'
        WHEN 14 THEN 'Joseph'
        WHEN 15 THEN 'Jessica'
        WHEN 16 THEN 'Thomas'
        WHEN 17 THEN 'Sarah'
        WHEN 18 THEN 'Charles'
        ELSE 'Karen'
    END || i,
    CASE (i % 15)
        WHEN 0 THEN 'Smith'
        WHEN 1 THEN 'Johnson'
        WHEN 2 THEN 'Williams'
        WHEN 3 THEN 'Brown'
        WHEN 4 THEN 'Jones'
        WHEN 5 THEN 'Garcia'
        WHEN 6 THEN 'Miller'
        WHEN 7 THEN 'Davis'
        WHEN 8 THEN 'Rodriguez'
        WHEN 9 THEN 'Martinez'
        WHEN 10 THEN 'Hernandez'
        WHEN 11 THEN 'Lopez'
        WHEN 12 THEN 'Gonzalez'
        WHEN 13 THEN 'Wilson'
        ELSE 'Anderson'
    END || (i / 100),
    (18 + (i % 62))::INTEGER
FROM generate_series(1, 10000) i;

-- Insert 100 tags
INSERT INTO tags (name, color)
SELECT 
    CASE (i % 20)
        WHEN 0 THEN 'technology'
        WHEN 1 THEN 'programming'
        WHEN 2 THEN 'database'
        WHEN 3 THEN 'rust'
        WHEN 4 THEN 'postgresql'
        WHEN 5 THEN 'performance'
        WHEN 6 THEN 'tutorial'
        WHEN 7 THEN 'beginner'
        WHEN 8 THEN 'advanced'
        WHEN 9 THEN 'tips'
        WHEN 10 THEN 'news'
        WHEN 11 THEN 'review'
        WHEN 12 THEN 'howto'
        WHEN 13 THEN 'guide'
        WHEN 14 THEN 'best-practices'
        WHEN 15 THEN 'architecture'
        WHEN 16 THEN 'design'
        WHEN 17 THEN 'testing'
        WHEN 18 THEN 'deployment'
        ELSE 'devops'
    END || '_' || i,
    '#' || lpad(to_hex(floor(random() * 16777215)::integer), 6, '0')
FROM generate_series(1, 100) i;

-- Insert 50,000 posts (5 posts per user average)
INSERT INTO posts (user_id, title, content, status, view_count)
SELECT 
    u.id,
    'Post Title ' || p.post_num || ' by ' || u.username,
    'This is the content for post number ' || p.post_num || '. ' ||
    'It contains multiple sentences to simulate realistic blog content. ' ||
    'The post discusses various topics related to technology and programming. ' ||
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit. ' ||
    'Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ' ||
    'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.',
    CASE (p.post_num % 10)
        WHEN 0 THEN 'draft'
        WHEN 1 THEN 'archived'
        ELSE 'published'
    END,
    (random() * 10000)::INTEGER
FROM users u
CROSS JOIN LATERAL (
    SELECT generate_series(1, 5) as post_num
) p
WHERE u.id IN (SELECT id FROM users ORDER BY created_at LIMIT 5000);

-- Insert 200,000 comments (4 comments per post average)
INSERT INTO comments (post_id, user_id, content)
SELECT 
    p.id,
    (SELECT id FROM users ORDER BY random() LIMIT 1),
    'This is comment ' || c.comment_num || ' on this post. ' ||
    'Great content! I really enjoyed reading this article. ' ||
    'Looking forward to more posts like this.'
FROM posts p
CROSS JOIN LATERAL (
    SELECT generate_series(1, 4) as comment_num
) c
WHERE p.id IN (SELECT id FROM posts ORDER BY created_at LIMIT 20000);

-- Link posts with tags (3 tags per post average)
INSERT INTO post_tags (post_id, tag_id)
SELECT DISTINCT ON (p.id, t.id)
    p.id,
    t.id
FROM posts p
CROSS JOIN LATERAL (
    SELECT id FROM tags ORDER BY random() LIMIT 3
) t
WHERE p.id IN (SELECT id FROM posts ORDER BY random() LIMIT 30000)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Statistics and Verification
-- ============================================================================

-- Update table statistics for query planner
ANALYZE users;
ANALYZE posts;
ANALYZE comments;
ANALYZE tags;
ANALYZE post_tags;

-- Display data counts
DO $$
DECLARE
    user_count INTEGER;
    post_count INTEGER;
    comment_count INTEGER;
    tag_count INTEGER;
    post_tag_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO post_count FROM posts;
    SELECT COUNT(*) INTO comment_count FROM comments;
    SELECT COUNT(*) INTO tag_count FROM tags;
    SELECT COUNT(*) INTO post_tag_count FROM post_tags;
    
    RAISE NOTICE 'Database initialized with:';
    RAISE NOTICE '  - % users', user_count;
    RAISE NOTICE '  - % posts', post_count;
    RAISE NOTICE '  - % comments', comment_count;
    RAISE NOTICE '  - % tags', tag_count;
    RAISE NOTICE '  - % post-tag relationships', post_tag_count;
END $$;
