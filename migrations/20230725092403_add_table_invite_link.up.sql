-- Add up migration script here
CREATE TABLE IF NOT EXISTS invite_link (
    user_id INTEGER NOT NULL,
    chat_id TEXT NOT NULL,
    link TEXT NOT NULL,
    created_at DATETIME NOT NULL
);
CREATE INDEX invite_link_user_id_chat_id_idx ON invite_link (user_id, chat_id);
