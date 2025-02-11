SHELL := /bin/bash
ENV=source .env &&
PROJECT_NAME=beancount

# The registry is presumed to be docker.io, which is the implicit default
DOCKER_ACCOUNT=jdevries3133
ifdef GITHUB_SHA
	TAG=$(GITHUB_SHA)
else
	TAG=$(shell git rev-parse HEAD)
endif
CONTAINER_QUALNAME=$(DOCKER_ACCOUNT)/$(PROJECT_NAME)
CONTAINER_EXACT_REF=$(DOCKER_ACCOUNT)/$(PROJECT_NAME):$(TAG)

.PHONY: build
.PHONY: check
.PHONY: setup
.PHONY: dev
.PHONY: bootstrap
.PHONY: deploy
.PHONY: _start-db
.PHONY: _stop-db
.PHONY: watch-db
.PHONY: shell-db
.PHONY: build-container
.PHONY: debug-container
.PHONY: push-container

check: setup
ifdef CI
	pnpm run build
endif
ifndef CI
	@# Locally, we want to ensure that `cargo sqlx prepare` was run, otherwise
	@# the build will fail in CI. So, we'll run an offline build as part of
	@# our checks
	SQLX_OFFLINE=true cargo build --features production
endif
	./scripts/lint_dbg.sh
	./scripts/lint_todo.sh
	cargo clippy --features production -- -D warnings
	cargo fmt --check
	cargo test

sqlx:
	cargo sqlx prepare -- --features production

build: setup
	pnpm run build
	cargo build --release

setup:
	./scripts/download_htmx.sh
ifdef CI
	npm i -g pnpm
endif
	[[ ! -d node_modules ]] \
		&& pnpm install \
		|| true
ifndef CI
	@# we only want the `.env` file locally in practice. We never run the app
	@# in CI (yet). The problem with having the `.env` file in CI is that
	@# sqlx will pickup on the `DATABASE_URL` environment variable and try
	@# to talk to a datbase that isn't there, causing compilation to fail.
	@# See also https://github.com/launchbadge/sqlx/blob/540baf7df55a372cb79d8636d02b1361a495b344/sqlx-cli/README.md#force-building-in-offline-mode
	[[ ! -f .env ]] && cp env-template .env || true
endif

dev: setup
	npx concurrently --names 'tailwind,cargo,stripe,proxy-prod-db' \
		'pnpm run dev' \
		"cargo watch -x 'run --features \"live_reload stripe use_stripe_test_instance localhost_base_url\"'" \
		'make proxy-stripe-webhook' \
		'make proxy-prod-db'

bootstrap: setup _stop-db
	SQLX_OFFLINE=true cargo build
	make _start-db
	@sleep 1  # give the DB time to startup
	@echo "===================================================================="
	@echo "Bootstrap complete! The app is running now, but you need to stop it"
	@echo "and run 'make dev' to get live-reloading started."
	@echo "===================================================================="
	$(ENV) ./target/debug/calcount

deploy:
ifdef CI
	terraform init
endif
	terraform apply -auto-approve

# Spawn a development database from a backup of the production database at
# $HOME/calcount-backup.sql
db-from-backup: _stop-db
	$(ENV) docker run \
        --name $(PROJECT_NAME) \
        -e POSTGRES_DATABASE="$$POSTGRES_DB" \
        -e POSTGRES_USER="$$POSTGRES_USER" \
        -e POSTGRES_PASSWORD="$$POSTGRES_PASSWORD" \
        -v $$HOME/calcount-backup.sql:/docker-entrypoint-initdb.d/initdb.sql \
        -p 5432:5432 \
        -d \
        postgres:15


_start-db:
	$(ENV) docker run \
        --name $(PROJECT_NAME) \
        -e POSTGRES_DB="$$POSTGRES_DB" \
        -e POSTGRES_USER="$$POSTGRES_USER" \
        -e POSTGRES_PASSWORD="$$POSTGRES_PASSWORD" \
        -p 5432:5432 \
        -d \
        postgres:15

_stop-db:
	docker kill $(PROJECT_NAME) || true
	docker rm $(PROJECT_NAME) || true

watch-db:
	docker logs -f $(PROJECT_NAME)

shell-db:
	$(ENV) PGPASSWORD=$$POSTGRES_PASSWORD \
		psql -U "$$POSTGRES_USER" -h 0.0.0.0 $$POSTGRES_DB

prod-shell-db:
	kubectl exec \
		-it \
		-n beancount \
		pod/db-postgresql-0 \
		-- /bin/sh -c 'psql postgresql://beancount:$$POSTGRES_PASSWORD@127.0.0.1:5432/beancount'

# Port-forward the production database to your local port 5433 (as to not
# collide with your local PostgreSQL server).
proxy-prod-db:
	kubectl -n beancount port-forward service/db-postgresql 5433:5432

proxy-stripe-webhook:
	stripe listen --forward-to localhost:8000/stripe-webhook

backup-prod:
	kubectl exec \
		-n $(PROJECT_NAME) \
		pod/db-postgresql-0 \
		-- /bin/sh -c "pg_dump postgresql://$(PROJECT_NAME):\$$POSTGRES_PASSWORD@127.0.0.1:5432/$(PROJECT_NAME)" \
		> ~/Desktop/$(PROJECT_NAME)_backups/backup-$(shell date '+%m-%d-%Y__%H:%M:%S').sql

build-container: setup
	pnpm run build
	rustup target add x86_64-unknown-linux-musl
	cargo build \
		--release \
		--target x86_64-unknown-linux-musl \
		--features production
	docker buildx build --load --platform linux/amd64 -t $(CONTAINER_EXACT_REF) .

# Run the above container locally, such that it can talk to the local
# PostgreSQL database launched by `make _start-db`. We expect here that the
# local database is already running and the container has already been built.
#
# Note: on macOS, you need to change the database host to "host.docker.internal"
# to allow the container to talk to the local PostgreSQL instance running inside
# Docker. On Linux, you can add the --net=host flag to the invocation of
# `docker run` below, to make PostgreSQL at localhost:5432 visible to the
# container.
debug-container:
	$(ENV) docker run \
		-e RUST_BACKTRACE=1 \
		-e DATABASE_URL="$$DATABASE_URL" \
		-e SESSION_SECRET="$$SESSION_SECRET" \
		-e OPENAI_API_KEY="$$OPENAI_API_KEY" \
		-e STRIPE_API_KEY="$$STRIPE_API_KEY" \
		-e STRIPE_WEBHOOK_SIGNING_SECRET="$$STRIPE_WEBHOOK_SIGNING_SECRET" \
		-e SMTP_EMAIL_USERNAME="$$SMTP_EMAIL_USERNAME" \
		-e SMTP_EMAIL_PASSWORD="$$SMTP_EMAIL_PASSWORD" \
		-p 8000:8000 \
		$(CONTAINER_EXACT_REF)

push-container: build-container
	docker push $(CONTAINER_EXACT_REF)

# I use this with vim `:read !make migration-cat` to view all migration files
# together in a single file.
migration-cat:
	find migrations | grep ".sql$$" | xargs cat
