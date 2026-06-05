# caglla-cli

Caglla.Travel の CLI 版。ローカル SQLite DB で旅行 (Trip) を管理します。

## セットアップ

```bash
cargo build
```

## 実行例

```bash
cargo run -- trip add "沖縄旅行"
cargo run -- trip list
cargo run -- trip show 1
cargo run -- trip update 1 --name "沖縄・瀬底旅行"
cargo run -- trip delete 1
```

## 品質チェック

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build
```
