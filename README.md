# Caglla.Travel CLI

Caglla.Travel のコマンドライン版です。旅行の計画を、ターミナルから管理できます。

現時点では **ローカルの SQLite データベース**（`caglla.db`）にデータを保存する CLI アプリです。Web 版やクラウド同期には未対応です。

## できること

- **Trip（旅行）** の登録・一覧・詳細・更新・削除
- **Itinerary（日程）** の登録・一覧・詳細・更新・削除
- 各予定への **開始時刻・所要時間・移動時間** の設定
- **Timeline（タイムライン）** による旅行の流れの表示
- **db reset** による開発用 DB 初期化

## 必要な環境

- [Rust](https://www.rust-lang.org/)（`cargo` が使えること）

## インストール

リポジトリをクローンしたあと、プロジェクト直下でビルドします。

```bash
cargo build
```

ビルドが成功すれば、`cargo run --` の後ろにコマンドを付けて実行できます。

## 品質チェック（make check）

コードの整形・静的解析・テスト・ビルドをまとめて確認できます。

```bash
make check
```

内部では次のコマンドを順番に実行します。

1. `cargo fmt --check` — コード整形の確認
2. `cargo clippy -- -D warnings` — 警告なしの静的解析
3. `cargo test` — ユニットテスト
4. `cargo build` — ビルド

その他の Make ターゲット:

| コマンド | 内容 |
|---|---|
| `make test` | テストのみ実行 |
| `make run` | `cargo run` を実行 |
| `make clean` | ビルド成果物を削除 |

## データベースについて

- DB ファイル名: `caglla.db`（プロジェクト直下に作成されます）
- 初回起動時に `trips` / `itinerary_items` テーブルが自動作成されます
- 既存の DB がある場合は、不足している列を自動で追加します（マイグレーション）

### DB 初期化（開発用）

**開発・動作確認用** のコマンドです。本番運用では使わないでください。

```bash
cargo run -- db reset
```

- `itinerary_items` のデータを全削除
- `trips` のデータを全削除
- テーブル定義は残す
- ID の採番（AUTOINCREMENT）をリセット

## Trip（旅行）の使い方

### 旅行を追加

```bash
cargo run -- trip add "沖縄旅行"
cargo run -- trip add "京都旅行" --start 2026-05-01 --end 2026-05-03
```

| オプション | 説明 |
|---|---|
| `name` | 旅行名（必須） |
| `--start` | 開始日（YYYY-MM-DD、任意） |
| `--end` | 終了日（YYYY-MM-DD、任意） |

### 一覧・詳細

```bash
cargo run -- trip list
cargo run -- trip show 1
```

### 更新・削除

```bash
cargo run -- trip update 1 --name "沖縄・石垣旅行"
cargo run -- trip update 1 --start 2026-04-26 --end 2026-04-30
cargo run -- trip delete 1
```

更新時は `--name` / `--start` / `--end` のうち、変更したい項目だけ指定します。

## Itinerary（日程）の使い方

日程は **Trip ID** に紐づきます。先に `trip add` で旅行を作成してください。

### 日程を追加

```bash
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 12:30 "昼食" --note "沖縄そば"
cargo run -- itinerary add 1 --day 1 "ホテルチェックイン" --order 99
```

| オプション | 説明 |
|---|---|
| `trip_id` | 旅行 ID（必須） |
| `--day` | 何日目か（必須） |
| `title` | 予定名（必須） |
| `--time` | 開始時刻（HH:MM、任意） |
| `--duration` | 所要時間（分、任意） |
| `--travel` | 次の予定までの移動時間（分、任意） |
| `--note` | メモ（任意） |
| `--order` | 並び順（任意、小さいほど先。時刻未定のときに便利） |

### 一覧・詳細

```bash
cargo run -- itinerary list 1
cargo run -- itinerary show 1
```

一覧は **日目 → 時刻 → 並び順** の順で表示されます。時刻がある予定が先、時刻未定が後です。

### 更新・削除

```bash
cargo run -- itinerary update 1 --time 09:30 --duration 120
cargo run -- itinerary update 1 --title "首里城公園" --travel 25
cargo run -- itinerary delete 1
```

## Timeline（タイムライン）の使い方

旅行の 1 日の流れを、時系列で見やすく表示します。

```bash
cargo run -- itinerary timeline 1
```

表示例（イメージ）:

```
Day 1

09:00 首里城
  所要時間: 90分
  終了予定: 10:30

  ↓ 移動 20分

10:50 国際通り
  所要時間: 60分
  終了予定: 11:50
```

- 時刻が設定されている予定: 開始時刻・所要時間・終了予定を表示
- 時刻が未設定の予定: `時刻: 未定` と表示（終了予定は表示しません）
- 移動時間がある場合: 次の予定の前に `↓ 移動 N分` を表示

## 開発用サンプルシナリオ

沖縄旅行の 1 日目を登録し、タイムラインで確認する例です。  
まず DB を空にしてから、順番に実行してください。

```bash
cargo run -- db reset
cargo run -- trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 10:50 --duration 60 --travel 15 "国際通り"
cargo run -- itinerary add 1 --day 1 --time 13:00 --duration 120 "ホテルチェックイン"
cargo run -- itinerary timeline 1
```

途中で登録内容を確認したい場合:

```bash
cargo run -- trip list
cargo run -- itinerary list 1
```

## プロジェクト構成（現時点）

```
caglla-cli/
├── src/main.rs    # アプリ本体（CLI・DB・テストをすべて含む）
├── Cargo.toml
├── Makefile
├── caglla.db      # ローカル DB（実行時に自動作成、git 管理外）
└── README.md
```

現時点では学習目的のため、`main.rs` 1 ファイルにまとめています。
