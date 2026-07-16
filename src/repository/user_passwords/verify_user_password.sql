SELECT
    p.`user_id` AS `id: DbUserId`,
    p.`psk`
FROM `user_passwords` AS p
WHERE p.`user_id` = ?
