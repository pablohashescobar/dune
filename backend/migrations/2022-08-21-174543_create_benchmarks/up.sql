-- Your SQL goes here
CREATE TABLE "benchmark" (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        title TEXT NOT NULL,
        subject TEXT NOT NULL,
        difficulty TEXT NOT NULL,
        creator_id UUID,
        git_url TEXT,
        max_cyclomatic_complex INTEGER NOT NULL DEFAULT 10
);