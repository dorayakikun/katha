use crate::domain::message::Usage;

const JPY_PER_USD: f64 = 150.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct UsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub has_unknown: bool,
    pub has_data: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CostSummary {
    pub usd: f64,
    pub has_unknown: bool,
    pub has_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Jpy,
}

impl Currency {
    pub fn toggle(self) -> Self {
        match self {
            Currency::Usd => Currency::Jpy,
            Currency::Jpy => Currency::Usd,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Jpy => "JPY",
        }
    }

    pub fn format_cost(self, usd: f64) -> String {
        match self {
            Currency::Usd => format!("${usd:.4}"),
            Currency::Jpy => format!("¥{:.0}", usd * JPY_PER_USD),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostRate {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

pub fn cost_rate_for_model(model: &str) -> Option<CostRate> {
    let model = model.to_lowercase();
    // Claude 4.x (Opus 4/4.5)
    if model.contains("claude-4-5-opus")
        || model.contains("claude-4-opus")
        || model.contains("opus-4-5")
        || model.contains("opus-4")
    {
        return Some(CostRate {
            input_per_million: 5.0,
            output_per_million: 25.0,
        });
    }
    // Claude 3.x
    if model.contains("claude-3-5-sonnet") || model.contains("claude-3-7-sonnet") {
        return Some(CostRate {
            input_per_million: 3.0,
            output_per_million: 15.0,
        });
    }
    if model.contains("claude-3-5-haiku") || model.contains("claude-3-haiku") {
        return Some(CostRate {
            input_per_million: 0.25,
            output_per_million: 1.25,
        });
    }
    if model.contains("claude-3-opus") {
        return Some(CostRate {
            input_per_million: 15.0,
            output_per_million: 75.0,
        });
    }
    if model.contains("claude-3-sonnet") {
        return Some(CostRate {
            input_per_million: 3.0,
            output_per_million: 15.0,
        });
    }
    // Codex / GPT-5.x
    if model.contains("gpt-5.2-pro") {
        return Some(CostRate {
            input_per_million: 21.0,
            output_per_million: 168.0,
        });
    }
    if model.contains("gpt-5-pro") {
        return Some(CostRate {
            input_per_million: 15.0,
            output_per_million: 120.0,
        });
    }
    if model.contains("gpt-5.2") || model.contains("gpt-5-2") {
        return Some(CostRate {
            input_per_million: 1.75,
            output_per_million: 14.0,
        });
    }
    if model.contains("gpt-5-mini") {
        return Some(CostRate {
            input_per_million: 0.25,
            output_per_million: 2.0,
        });
    }
    if model.contains("gpt-5-nano") {
        return Some(CostRate {
            input_per_million: 0.05,
            output_per_million: 0.4,
        });
    }
    if model.contains("gpt-5") {
        return Some(CostRate {
            input_per_million: 1.25,
            output_per_million: 10.0,
        });
    }
    None
}

pub fn estimate_cost_usd(model: &str, usage: &Usage) -> Option<f64> {
    let rate = cost_rate_for_model(model)?;
    let input_tokens = usage.total_input_tokens() as f64;
    let output_tokens = usage.total_output_tokens() as f64;
    let input_cost = input_tokens * rate.input_per_million / 1_000_000.0;
    let output_cost = output_tokens * rate.output_per_million / 1_000_000.0;
    Some(input_cost + output_cost)
}

pub fn format_tokens(count: u64) -> String {
    let s = count.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let mut chars = s.chars().rev().enumerate().peekable();

    while let Some((i, ch)) = chars.next() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_rate_for_model_variants() {
        let cases = [
            (
                "claude-4-5-opus",
                CostRate {
                    input_per_million: 5.0,
                    output_per_million: 25.0,
                },
            ),
            (
                "claude-4-opus",
                CostRate {
                    input_per_million: 5.0,
                    output_per_million: 25.0,
                },
            ),
            (
                "gpt-5.2-pro",
                CostRate {
                    input_per_million: 21.0,
                    output_per_million: 168.0,
                },
            ),
            (
                "gpt-5-pro",
                CostRate {
                    input_per_million: 15.0,
                    output_per_million: 120.0,
                },
            ),
            (
                "gpt-5.2",
                CostRate {
                    input_per_million: 1.75,
                    output_per_million: 14.0,
                },
            ),
            (
                "gpt-5-2",
                CostRate {
                    input_per_million: 1.75,
                    output_per_million: 14.0,
                },
            ),
            (
                "gpt-5",
                CostRate {
                    input_per_million: 1.25,
                    output_per_million: 10.0,
                },
            ),
            (
                "gpt-5-mini",
                CostRate {
                    input_per_million: 0.25,
                    output_per_million: 2.0,
                },
            ),
            (
                "gpt-5-nano",
                CostRate {
                    input_per_million: 0.05,
                    output_per_million: 0.4,
                },
            ),
            (
                "claude-3-5-sonnet",
                CostRate {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                },
            ),
            (
                "claude-3-7-sonnet",
                CostRate {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                },
            ),
            (
                "claude-3-5-haiku",
                CostRate {
                    input_per_million: 0.25,
                    output_per_million: 1.25,
                },
            ),
            (
                "claude-3-haiku",
                CostRate {
                    input_per_million: 0.25,
                    output_per_million: 1.25,
                },
            ),
            (
                "claude-3-opus",
                CostRate {
                    input_per_million: 15.0,
                    output_per_million: 75.0,
                },
            ),
            (
                "claude-3-sonnet",
                CostRate {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                },
            ),
        ];

        for (model, expected) in cases {
            let actual = cost_rate_for_model(model);
            assert_eq!(actual, Some(expected));
        }
    }
}
