CREATE TABLE IF NOT EXISTS `users` (
    `id` BINARY(16) NOT NULL PRIMARY KEY,
    `display_id` VARCHAR(32) NOT NULL UNIQUE,
    `name` VARCHAR(32) NOT NULL
);

CREATE TABLE IF NOT EXISTS `user_passwords` (
    `user_id` BINARY(16) NOT NULL PRIMARY KEY,
    `psk` VARCHAR(128) NOT NULL
);

-- session sotre `user_sessions` is created via async-sqlx-session:
-- https://github.com/jbr/async-sqlx-session/blob/06a3abb8941edaea3d3e8133c30ee16231914a25/src/mysql.rs#L148-L168
