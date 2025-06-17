use sea_orm::prelude::Decimal;

pub(crate) fn to_decimal_2_digits(value: f64) -> Decimal {
    Decimal::from_i128_with_scale((value * 100.0).round() as i128, 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_decimal_2_digits() {
        let values = [1.2301, 2.3498, 300_f64];
        let expects = ["1.23", "2.35", "300.00"];

        for (i, value) in values.iter().enumerate() {
            let result = to_decimal_2_digits(*value);
            assert_eq!(result.to_string(), expects[i]);
        }
    }
}
