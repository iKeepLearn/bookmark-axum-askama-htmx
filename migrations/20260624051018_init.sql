-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    user_id  UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    role     TEXT NOT NULL DEFAULT 'guest'
);

CREATE TABLE IF NOT EXISTS categories (
    id         INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE TABLE IF NOT EXISTS tags (
    id         INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE TABLE IF NOT EXISTS bookmarks (
    id          INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    category_id INT NOT NULL,
    title       TEXT NOT NULL,
    url         TEXT NOT NULL UNIQUE,
    cover_image TEXT NOT NULL,
    "desc"      TEXT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_bookmarks_category
        FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id INT NOT NULL,
    tag_id     INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (bookmark_id, tag_id),

    CONSTRAINT fk_bookmark_tags_bookmark
        FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,

    CONSTRAINT fk_bookmark_tags_tag
        FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_category_id ON bookmarks(category_id);


INSERT INTO users (user_id, username, password, role)
VALUES
    ('ddf8994f-d522-4659-8d02-c1d479057be7',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$S6A7fJxh3oAV3DqH9iq3fQ$tuDk8xQOz1K2IZ5dTdDhjemSVUsuTpjTW7hNZsyt/HQ',
    'admin'),
    ('ddf8994f-d522-4659-8d02-c1d479057be8',
    'guest',
    '$argon2id$v=19$m=15000,t=2,p=1$Hq56v2qebnVxwALx/bdLIw$kbux8VrxBt8M9cxYoSVy4Pr8dQ9pQinOTD9w6T5u1e8',
    'guest')
ON CONFLICT (username) DO NOTHING;
