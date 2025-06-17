pub(crate) fn format_j_reit_corporation_name(name: &str, is_delisted: bool) -> String {
    if is_delisted {
        format!("{} (上場廃止)", name)
    } else {
        name.to_string()
    }
}

pub(crate) fn format_native_date(native_date: &chrono::NaiveDate) -> String {
    native_date.format("%Y/%m/%d").to_string()
}

pub(crate) fn format_integer(number: i64) -> String {
    number.to_string()
}

pub(crate) fn format_float(number: f64) -> String {
    format!("{:.2}", number)
}

pub(crate) fn format_completed_year_month(year: &Option<i64>, month: &Option<i64>) -> String {
    if let Some(year) = year {
        if let Some(month) = month {
            // excel側で 2022/01 → Jan/22 と表示されてしまうため末尾の/1を追加
            format!("{}/{:02}/01", year, month)
        } else {
            // 竣工年しかわからない場合は、月を'  'で表示する
            format!("{}   ", year)
        }
    } else {
        "".to_string()
    }
}

const SQUARE_METER_TO_TSUBO: f64 = 0.3025;

pub(crate) fn format_area_f64_in_square_meter(area: f64) -> String {
    format_float(area_f64_in_square_meter(area))
}

pub(crate) fn area_f64_in_square_meter(area: f64) -> f64 {
    area / SQUARE_METER_TO_TSUBO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_j_reit_corporation_name() {
        assert_eq!(
            format_j_reit_corporation_name("東京建物株式会社", false),
            "東京建物株式会社"
        );
        assert_eq!(
            format_j_reit_corporation_name("東京建物株式会社", true),
            "東京建物株式会社 (上場廃止)"
        );
    }

    #[test]
    fn test_format_integer() {
        assert_eq!(format_integer(100), "100");
        assert_eq!(format_integer(1005010), "1005010");
    }

    #[test]
    fn test_format_completed_year_month() {
        assert_eq!(
            format_completed_year_month(&Some(2021), &Some(1)),
            "2021/01/01"
        );
        assert_eq!(format_completed_year_month(&Some(2021), &None), "2021   ");
        assert_eq!(format_completed_year_month(&None, &Some(1)), "");
        assert_eq!(format_completed_year_month(&None, &None), "");
    }

    #[test]
    fn test_format_float() {
        assert_eq!(format_float(100.0), "100.00");
        assert_eq!(format_float(100.5010), "100.50");
        assert_eq!(format_float(100.501), "100.50");
        assert_eq!(format_float(100.5), "100.50");
        assert_eq!(format_float(100.55), "100.55");
        assert_eq!(format_float(100.555), "100.56");
        assert_eq!(format_float(100.5555), "100.56");
        assert_eq!(format_float(1000000.0), "1000000.00");
        assert_eq!(format_float(1000000.1), "1000000.10");
        assert_eq!(format_float(0.1), "0.10");
    }

    #[test]
    fn test_format_area_f64_in_square_meter() {
        assert_eq!(format_area_f64_in_square_meter(123.4), "407.93");
        assert_eq!(format_area_f64_in_square_meter(0.0), "0.00");
    }
}
