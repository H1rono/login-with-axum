SELECT
    u.`id` AS `id: DbUserId`,
    u.`display_id`,
    u.`name`
FROM `users` AS u
WHERE u.`id` = ?
