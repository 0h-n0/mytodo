# MyTODO

* (WIP) frontend/React
* (DONE) backend/Rust(Axum)
  * Sea-Orm
* (DONW) database/PostgreSQL
* (WIP) cloud/terraform
  * AWS

## Prerequisites

* [rust](https://www.rust-lang.org/tools/install)
* [nodejs](https://nodejs.org/en/download/)
* [docker](https://docs.docker.com/engine/install/)
* [docker-compose](https://docs.docker.com/compose/install/)
* [terraform](https://www.terraform.io/)

## Backend API

see `openapi.yaml`

## table schema

```sql
$ cargo install sea-orm-cli
```

## Setup

### Database

```bash
$ docker-compose up -d
```

```bash
$ sudo apt install postgresql-client
$ psql -h localhost -p 5432 -U postgres
```

```sql
$ create database todo;
```

```bash
$ cd backend
$ cargo install sea-orm-cli
$ sea-orm-cli migrate init
```

#### execute migration

```bash
sea-orm-cli migrate refresh  
```

#### generate entity

```bash
$ sea-orm-cli generate entity -u ${DATABASE_URL} -o entity/src
```

### Note

```shell
backend/entity/src/
├── lib.rs
├── mod.rs
├── prelude.rs
└── todo.rs  ## <---
```

```rust
// todo.rs
// add Serialize Derive, because of the error
// * https://zenn.dev/hitochan777/scraps/13e8c8af011c7d
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "todo")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub text: String,
    pub completed: bool,
}
```


## Reference

* Backend
  * https://github.com/tokio-rs/axum/blob/main/examples/todos/src/main.rs
* Sea-Orm
  * https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example