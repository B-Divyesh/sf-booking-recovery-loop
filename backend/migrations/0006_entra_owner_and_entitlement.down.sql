DROP TABLE practice_entitlements;
DROP INDEX practices_owner_oid_idx;
ALTER TABLE practices DROP COLUMN owner_oid;
