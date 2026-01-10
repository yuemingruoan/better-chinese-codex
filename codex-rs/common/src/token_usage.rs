use codex_protocol::protocol::TokenUsage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenUsageSplit {
    pub prior: TokenUsage,
    pub last: TokenUsage,
}

pub fn split_total_and_last(total: &TokenUsage, last: &TokenUsage) -> TokenUsageSplit {
    let total = normalized_usage(total);
    let last = normalized_usage(last);

    let prior = TokenUsage {
        input_tokens: total.input_tokens.saturating_sub(last.input_tokens),
        cached_input_tokens: total
            .cached_input_tokens
            .saturating_sub(last.cached_input_tokens),
        output_tokens: total.output_tokens.saturating_sub(last.output_tokens),
        reasoning_output_tokens: total
            .reasoning_output_tokens
            .saturating_sub(last.reasoning_output_tokens),
        total_tokens: total.total_tokens.saturating_sub(last.total_tokens),
    };

    TokenUsageSplit { prior, last }
}

pub fn format_token_count_compact(value: i64) -> String {
    let value = value.max(0);
    if value < 1_000 {
        return value.to_string();
    }

    let (scaled, suffix) = if value >= 1_000_000 {
        (value as f64 / 1_000_000.0, "m")
    } else {
        (value as f64 / 1_000.0, "k")
    };

    let mut formatted = format!("{scaled:.1}");
    if formatted.ends_with(".0") {
        formatted.truncate(formatted.len() - 2);
    }

    format!("{formatted}{suffix}")
}

fn normalized_usage(usage: &TokenUsage) -> TokenUsage {
    TokenUsage {
        input_tokens: usage.input_tokens.max(0),
        cached_input_tokens: usage.cached_input_tokens.max(0),
        output_tokens: usage.output_tokens.max(0),
        reasoning_output_tokens: usage.reasoning_output_tokens.max(0),
        total_tokens: usage.total_tokens.max(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_token_count_compact_scales_k_and_m() {
        assert_eq!(format_token_count_compact(0), "0");
        assert_eq!(format_token_count_compact(999), "999");
        assert_eq!(format_token_count_compact(1_000), "1k");
        assert_eq!(format_token_count_compact(1_100), "1.1k");
        assert_eq!(format_token_count_compact(1_000_000), "1m");
        assert_eq!(format_token_count_compact(3_200_000), "3.2m");
        assert_eq!(format_token_count_compact(-5), "0");
    }

    #[test]
    fn split_total_and_last_saturates_per_field() {
        let total = TokenUsage {
            input_tokens: 100,
            cached_input_tokens: 20,
            output_tokens: 40,
            reasoning_output_tokens: 5,
            total_tokens: 165,
        };
        let last = TokenUsage {
            input_tokens: 10,
            cached_input_tokens: 2,
            output_tokens: 4,
            reasoning_output_tokens: 1,
            total_tokens: 17,
        };
        let split = split_total_and_last(&total, &last);

        assert_eq!(
            split.prior,
            TokenUsage {
                input_tokens: 90,
                cached_input_tokens: 18,
                output_tokens: 36,
                reasoning_output_tokens: 4,
                total_tokens: 148,
            }
        );
        assert_eq!(split.last, last);
    }
}
