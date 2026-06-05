use anyhow::Result;
use serde::{Deserialize, Serialize};

/// trips テーブルの1行分のデータ
#[derive(Clone, Serialize, Deserialize)]
pub struct Trip {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 日程カテゴリ（定義済みのみ受け付ける）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItineraryCategory {
    Flight,
    Hotel,
    Restaurant,
    Activity,
    Transport,
    Shopping,
    Beach,
    Museum,
}

impl ItineraryCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flight => "flight",
            Self::Hotel => "hotel",
            Self::Restaurant => "restaurant",
            Self::Activity => "activity",
            Self::Transport => "transport",
            Self::Shopping => "shopping",
            Self::Beach => "beach",
            Self::Museum => "museum",
        }
    }
}

const ITINERARY_CATEGORY_VALUES: &[&str] = &[
    "flight",
    "hotel",
    "restaurant",
    "activity",
    "transport",
    "shopping",
    "beach",
    "museum",
];

/// CLI 文字列からカテゴリを変換する（`none` は解除用のためここでは受け付けない）
pub(crate) fn parse_itinerary_category(s: &str) -> Result<ItineraryCategory> {
    match s {
        "flight" => Ok(ItineraryCategory::Flight),
        "hotel" => Ok(ItineraryCategory::Hotel),
        "restaurant" => Ok(ItineraryCategory::Restaurant),
        "activity" => Ok(ItineraryCategory::Activity),
        "transport" => Ok(ItineraryCategory::Transport),
        "shopping" => Ok(ItineraryCategory::Shopping),
        "beach" => Ok(ItineraryCategory::Beach),
        "museum" => Ok(ItineraryCategory::Museum),
        _ => anyhow::bail!(
            "不正なカテゴリです: {s}. 有効な値: {}",
            ITINERARY_CATEGORY_VALUES.join(", ")
        ),
    }
}

/// itinerary_items テーブルの1行分のデータ
#[derive(Clone, Serialize, Deserialize)]
pub struct ItineraryItem {
    pub id: i64,
    pub trip_id: i64,
    pub day: i64,
    pub title: String,
    pub note: Option<String>,
    pub start_time: Option<String>,
    pub sort_order: i64,
    pub duration_minutes: Option<i64>,
    pub travel_minutes: Option<i64>,
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ItineraryCategory>,
    pub created_at: String,
    pub updated_at: String,
}

/// checklist_items テーブルの1行分のデータ
#[derive(Clone)]
pub struct ChecklistItem {
    pub id: i64,
    pub trip_id: i64,
    pub title: String,
    pub is_done: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// trip export 用の JSON 構造
#[derive(Serialize, Deserialize)]
pub struct TripExport {
    pub trip: Trip,
    pub itinerary_items: Vec<ItineraryItem>,
}

#[cfg(test)]
mod tests {
    use crate::models::parse_itinerary_category;

    #[test]
    fn test_parse_invalid_itinerary_category() {
        assert!(parse_itinerary_category("invalid").is_err());
        assert!(parse_itinerary_category("lodging").is_err());
    }
}
