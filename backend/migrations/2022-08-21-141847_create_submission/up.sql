-- Your SQL goes here
CREATE TABLE "submission" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    language TEXT NOT NULL,
    code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP,
    user_id UUID NOT NULL,
    status TEXT NOT NULL,
    benchmark_id UUID,
    stdout TEXT,
    stderr TEXT,
    exec_duration INTEGER NOT NULL DEFAULT 0,
    message TEXT,
    error TEXT,
    lint_score INTEGER DEFAULT 0,
    quality_score INTEGER DEFAULT 0,
    mem_usage INTEGER NOT NULL DEFAULT 0,
    code_hash TEXT,
    cyclomatic_complexity INTEGER NOT NULL DEFAULT 0
);