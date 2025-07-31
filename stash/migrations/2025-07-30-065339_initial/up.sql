CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created TEXT NOT NULL
);

CREATE UNIQUE INDEX ix_tags_name ON tags(name);

CREATE TABLE file_contents (
    id INTEGER PRIMARY KEY,
    size INTEGER NOT NULL,
    hash TEXT NOT NULL,
    uploader TEXT NOT NULL,
    created TEXT NOT NULL
);

CREATE UNIQUE INDEX ix_file_contents_hash ON file_contents(hash);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    content_id INTEGER NOT NULL,
    uploader TEXT NOT NULL,
    created TEXT NOT NULL,
    FOREIGN KEY (content_id) REFERENCES file_contents(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ix_files_name ON files(name);

CREATE TABLE file_tags (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ix_file_tags_file_tag ON file_tags (file_id, tag_id);
