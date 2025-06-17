# rust-backend

# DB から SQL の用のコードを自動生成する

以下のコマンドで、コンテナで動く MySQL のスキーマから `./sql-entities/src/generated` にコードを生成する。

```sh
docker compose exec backend sea-orm-cli generate entity -u mysql://root:password@mysql/test -o ./sql-entities/src/generated
```


# GraphQL のスキーマを生成する

以下のコマンドで、コンテナで動く Rust のサーバーから `./frontend/graphql/j-reit/schema.graphql` にスキーマを生成する。

```sh
docker compose exec backend sh -c 'cargo run -p graphql > ../frontend/graphql/j-reit/schema.graphql'
```
