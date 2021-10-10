CREATE TABLE IF NOT EXISTS hello (
	id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
	created_at datetime(6) NOT NULL,
	updated_at datetime(6) NOT NULL,
	deleted_at datetime(6) DEFAULT NULL,
	name VARCHAR(256) NOT NULL,
	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS hello_status (
	id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
	hello_id BIGINT UNSIGNED NOT NULL,
	created_at datetime(6) NOT NULL,
	updated_at datetime(6) NOT NULL,
	deleted_at datetime(6) DEFAULT NULL,
	message VARCHAR(256) NOT NULL,
	PRIMARY KEY (id)
);
