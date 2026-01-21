CREATE TYPE Link AS (
    href TEXT,
    rel TEXT,
    type TEXT,
    hreflang TEXT,
    title TEXT,
    length BIGINT
);

CREATE TYPE Response AS ENUM (
    'raw',
    'document'
);

CREATE TYPE JobType AS ENUM (
    'process'
);

CREATE TYPE StatusCode AS ENUM (
    'accepted',
    'running',
    'successful',
    'failed',
    'dismissed'
);

CREATE TABLE jobs (
    -- StatusInfo
    job_id TEXT PRIMARY KEY,
    process_id TEXT,
    type JobType NOT NULL,
    status StatusCode NOT NULL,
    message TEXT,
    created TIMESTAMPTZ NOT NULL,
    finished TIMESTAMPTZ,
    updated TIMESTAMPTZ NOT NULL,
    progress SMALLINT CHECK (progress >= 0 AND progress <= 100),
    links Link[] NOT NULL CHECK (array_position(links, NULL) IS NULL) DEFAULT ARRAY[]::Link[],
    -- Response
    response Response NOT NULL,
    -- Results
    results JSONB, -- TODO: define structure
    -- User
    user_id UUID NOT NULL
);
