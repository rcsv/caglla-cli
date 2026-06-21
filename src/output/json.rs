use anyhow::{Context, Result};

/// 値を pretty JSON で標準出力する
pub(crate) fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("JSON の生成に失敗しました")?;
    println!("{json}");
    Ok(())
}
