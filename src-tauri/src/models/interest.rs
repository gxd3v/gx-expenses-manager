pub const INTEREST_TAX_RATE: f64 = 0.28;

#[derive(Debug, Clone, PartialEq)]
pub struct InterestTier {
    pub min_balance: i64,
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interest {
    pub period_months: u32,
    pub tiers: Vec<InterestTier>,
}

impl Interest {
    pub fn rate_for(&self, balance: i64) -> f64 {
        self.tiers
            .iter()
            .filter(|tier| balance >= tier.min_balance)
            .max_by_key(|tier| tier.min_balance)
            .map_or(0.0, |tier| tier.rate)
    }

    pub fn monthly_gross(&self, balance: i64) -> f64 {
        balance.max(0) as f64 * self.rate_for(balance) / 100.0 / 12.0
    }

    pub fn net_per_period(&self, balance: i64) -> i64 {
        net(self.monthly_gross(balance) * f64::from(self.period_months))
    }
}

pub fn net(gross: f64) -> i64 {
    (gross * (1.0 - INTEREST_TAX_RATE)).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiered() -> Interest {
        Interest {
            period_months: 3,
            tiers: vec![
                InterestTier {
                    min_balance: 0,
                    rate: 1.0,
                },
                InterestTier {
                    min_balance: 500_000,
                    rate: 1.25,
                },
                InterestTier {
                    min_balance: 5_000_000,
                    rate: 1.5,
                },
            ],
        }
    }

    #[test]
    fn whole_balance_earns_the_tier_rate() {
        let interest = tiered();
        assert_eq!(interest.rate_for(100_000), 1.0);
        assert_eq!(interest.rate_for(600_000), 1.25);
        assert_eq!(interest.rate_for(6_000_000), 1.5);
        assert_eq!(interest.rate_for(-10), 0.0);
    }

    #[test]
    fn net_interest_per_period_discounts_tax() {
        let interest = tiered();
        assert_eq!(interest.net_per_period(1_200_000), 2_700);
        assert_eq!(interest.net_per_period(-5_000), 0);
    }
}
