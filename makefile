
# Define the custom database URL (adjust as needed)
DATABASE_URL ?= "mysql://user:secret@localhost:3307/actix_rust"
DATABASE_TEST_URL ?= "mysql://user:secret@localhost:3307/actix_test"

.PHONY: install dev-install revert_test_migrations revert_migrations migrate_test dev_migrate test_migrate

install: 
	cargo add actix-web --features "openssl"
	cargo add openssl
	cargo add actix-cors
	cargo add tokio --features "full"
	cargo add log
	cargo add serde --features "derive"
	cargo add serde_json
	cargo add chrono --features "serde"
	cargo add env_logger
	cargo add dotenv
	cargo add uuid --features "serde v4"
	cargo add sqlx --features "runtime-async-std-native-tls mysql chrono uuid json"
	cargo add lettre
	cargo add lettre_email
	cargo add rand
	cargo add rand_core
	cargo add utoipa --features "actix_extras chrono uuid"
	cargo add utoipa-swagger-ui --features "actix-web"
	cargo add utoipa-rapidoc -F "actix-web"
	cargo add utoipa-redoc -F "actix-web"
	cargo add sha256
	cargo add jsonwebtoken
	cargo add twilio
	cargo add actix-web-lab
	cargo add regex
	cargo add humantime
	cargo add csv
	cargo add actix-multipart
	cargo add sanitize-filename
	cargo add lazy_static
	cargo add config --features "json"
	cargo add validator --features "derive"
	cargo add fake
	cargo add futures-util
	cargo add base32
	cargo add totp-rs
	cargo add async-trait
	cargo add actix-web-actors
	cargo add actix

dev-install:
	cargo add sqlx --dev --features "runtime-async-std-native-tls sqlite mysql chrono uuid json"
	cargo add tokio --dev --features "full"
	cargo add --dev mockall

revert_test_migrations:
	bash migrations/revert_all_migrations.sh ${DATABASE_TEST_URL}

revert_migrations:
	bash migrations/revert_all_migrations.sh ${DATABASE_URL}

migrate_test:
	sqlx migrate run --database-url ${DATABASE_TEST_URL}

migrate:
	sqlx migrate run --database-url ${DATABASE_URL}

dev_migrate:
	make revert_migrations || make migrate
test_migrate: 
	make revert_test_migrations || make migrate_test
