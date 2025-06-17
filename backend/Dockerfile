# cargo-chef の公式イメージを使う
FROM lukemathwalker/cargo-chef:latest-rust-1.84 AS chef
WORKDIR /app

# 依存関係を整理したファイル recipe.json を出力する。
# ソースツリーに変更があると再ビルドされるが、recipe.json に変更がなければ
# ボトルネック部分（依存パッケージのビルド）には変更が入らないので、遅くならない。
FROM chef AS planner
COPY ./backend .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# private repository にある crate をダウンロードするために
# GitHub のトークンを使うように設定する。
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN --mount=type=secret,id=GITHUB_TOKEN echo https://git:$(cat /run/secrets/GITHUB_TOKEN)@github.com > /root/credential
RUN git config --global credential.helper 'store --file /root/credential'

# recipe.json に基づいて dependencies をビルドする。
# recipe.json に変更がなければここまでキャッシュされている。
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# アプリケーションをビルドする
COPY ./backend .
COPY ./graphql/data-hub/schema.gql ./rest/src/schemas/data-hub/schema.gql
RUN cargo build --workspace --release

# プロダクション用の Docker イメージをビルドする
FROM gcr.io/distroless/cc-debian12 AS production
COPY --from=builder /app/target/release/rust-backend /usr/bin
COPY --from=builder /app/target/release/internal-api-server /usr/bin

CMD [ "rust-backend" ]
