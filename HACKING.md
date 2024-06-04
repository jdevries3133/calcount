# Getting Started

To run the project locally, you need the following CLI tools:

- [docker CLI](https://docs.docker.com/engine/reference/commandline/cli/)
- [cargo](https://rustup.rs/)
- [pnpm](https://pnpm.io/)
- [concurrently](https://www.npmjs.com/package/concurrently)
- [cURL](https://curl.se/)
- [Make](https://formulae.brew.sh/formula/make)

The following ports also must be free on your machine:

- `5432` for PostgreSQL
- `8000` for this application

## `.env` File

The `env-template` file shows environment variables that we expect. Note that this file is also used for bootstrapping a local PostgreSQL database, so you can keep the DB credentials and connection string the same, and a new DB will be created with those credentials.

## Bootstrapping & Developing 

You will need to bootstrap the app and database by performing on offline
compilation using `./sqlx-data.json` -- there's a handy make rule to get you
started;

```
make bootstrap
```

After running the bootstrap rule, the app will be running, but it won't
live-reload. To run the typical dev scripts, stop the app and run the dev rule:

```
make dev
```

You can run unit-tests with cargo:

```
cargo test
```

There are some utilities in the Makefile for working with the database. In
particular:

```
make shell-db  # attach to an interactive PostgreSQL shell inside the DB
make watch-db  # live-tail the database logs
```

Additionally, there is a rule for running CI just like it runs in CI!

```
make check
```

# Other Database Options

Of course, the application will happily converse with any PostgreSQL instance.
You can easily direct the program to your PostgreSQL instance of your choosing
by simply changing the `.env` file. Note that the `.env` file is created by
copying `env-template` the first time you run `make dev`. Naturally, it contains
other handy config levers.

Note that the application only uses `DATABASE_URL`. The rest of the `POSTGRES_*`
environment variables are only used in the Makefile and passed to the PostgreSQL
Docker container during startup.

# Contributing & License

Keep in mind the notes in [the contribution guide](../CONTRIBUTING.md), and
[license](../LICENSE). This project is open source but _not_ free, but the tech
stack behind the website is free and open source, available in the [PHAT stack
repo](https://github.com/jdevries3133/phat_stack).

In general, my intent is to open source any components of this project in the
tech stack / infrastructure layer, so if there are some new shiny goodies in
this repository that are missing from PHAT stack, feel free to open a PR into
PHAT stack to port them over.

The primary intent of open sourcing this repository is for it to be a learning
resource, and for transparency for our users.
