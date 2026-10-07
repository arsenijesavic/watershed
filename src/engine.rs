//! Month-by-month projection.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::goals::{Affordability, Goal};
use crate::types::{Balance, CashEvent, Cents, MonthlyCosts};

/// Everything the projection needs.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scenario {
    pub balance: Balance,
    pub costs: MonthlyCosts,
    pub events: Vec<CashEvent>,
    pub goals: Vec<Goal>,
    /// Number of months to project.
    pub months: usize,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MonthRecord {
    /// 1-based month index.
    pub month: usize,
    pub opening: Cents,
    pub income: Cents,
    /// Fixed costs plus event expenses.
    pub expenses: Cents,
    pub closing: Cents,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Report {
    pub timeline: Vec<MonthRecord>,
    /// Months the spendable balance lasts with no income; `None` when there are no fixed costs.
    pub runway_months: Option<f64>,
    /// First month the closing balance falls below the safety floor.
    pub first_breach_month: Option<usize>,
    /// Lowest balance seen, including the opening balance.
    pub low_point: Cents,
    /// Month of `low_point`; 0 means the opening balance.
    pub low_point_month: usize,
    /// Affordability of each goal, in the order given.
    pub goals: Vec<(String, Affordability)>,
    /// IDs of events whose cash lands outside the projection, so they were not applied.
    pub skipped_events: Vec<String>,
}

pub fn simulate(scenario: &Scenario) -> Report {
    let start = scenario.balance.total();
    let floor = scenario.balance.safety_floor;
    let fixed = scenario.costs.total();

    let mut timeline = Vec::with_capacity(scenario.months);
    let mut balance = start;
    for month in 1..=scenario.months {
        let (mut income, mut expenses) = (0, fixed);
        for event in scenario.events.iter().filter(|e| e.cash_month() == month) {
            match event.cash_delta() {
                delta if delta >= 0 => income += delta,
                delta => expenses -= delta,
            }
        }
        let opening = balance;
        balance += income - expenses;
        timeline.push(MonthRecord {
            month,
            opening,
            income,
            expenses,
            closing: balance,
        });
    }

    let (low_point_month, low_point) = timeline
        .iter()
        .map(|r| (r.month, r.closing))
        .filter(|&(_, closing)| closing < start)
        .min_by_key(|&(_, closing)| closing)
        .unwrap_or((0, start));

    Report {
        runway_months: (fixed > 0).then(|| scenario.balance.spendable() as f64 / fixed as f64),
        first_breach_month: timeline.iter().find(|r| r.closing < floor).map(|r| r.month),
        low_point,
        low_point_month,
        goals: scenario
            .goals
            .iter()
            .map(|goal| {
                (
                    goal.id.clone(),
                    affordability(goal, floor, start, &timeline),
                )
            })
            .collect(),
        skipped_events: scenario
            .events
            .iter()
            .filter(|e| !(1..=scenario.months).contains(&e.cash_month()))
            .map(|e| e.id.clone())
            .collect(),
        timeline,
    }
}

fn affordability(
    goal: &Goal,
    floor: Cents,
    start: Cents,
    timeline: &[MonthRecord],
) -> Affordability {
    let required = floor + goal.safety_buffer + goal.capital_needed();
    if start >= required {
        return Affordability::Now;
    }
    if let Some(record) = timeline.iter().find(|r| r.closing >= required) {
        return Affordability::FromMonth(record.month);
    }
    let peak = timeline.iter().map(|r| r.closing).fold(start, Cents::max);
    Affordability::NotInHorizon {
        shortfall: required - peak,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goals::GoalCost;
    use crate::types::{Confidence, EventKind};

    fn scenario(cash: Cents, floor: Cents, monthly: Cents, months: usize) -> Scenario {
        Scenario {
            balance: Balance {
                cash,
                savings: 0,
                safety_floor: floor,
            },
            costs: MonthlyCosts {
                living: monthly,
                studio: 0,
            },
            events: vec![],
            goals: vec![],
            months,
        }
    }

    fn income(id: &str, gross: Cents, month: usize) -> CashEvent {
        CashEvent {
            id: id.into(),
            label: id.into(),
            month,
            kind: EventKind::Income {
                gross,
                tax_rate: 0.0,
                payment_delay_months: 0,
                confidence: Confidence::Locked,
            },
        }
    }

    fn goal(cost: Cents) -> Goal {
        Goal {
            id: "g".into(),
            label: "g".into(),
            cost: GoalCost::OneTime(cost),
            safety_buffer: 0,
        }
    }

    #[test]
    fn detects_first_breach_and_low_point() {
        let report = simulate(&scenario(10_000, 5_000, 3_000, 3));
        assert_eq!(report.first_breach_month, Some(2));
        assert_eq!((report.low_point_month, report.low_point), (3, 1_000));
        assert_eq!(report.runway_months, Some(5_000.0 / 3_000.0));
    }

    #[test]
    fn income_is_taxed_weighted_and_delayed() {
        let event = CashEvent {
            kind: EventKind::Income {
                gross: 10_000,
                tax_rate: 0.25,
                payment_delay_months: 1,
                confidence: Confidence::InDiscussion,
            },
            ..income("x", 0, 2)
        };
        assert_eq!(event.cash_month(), 3);
        assert_eq!(event.cash_delta(), 3_750);
    }

    #[test]
    fn goal_becomes_affordable_after_income() {
        let mut s = scenario(10_000, 0, 1_000, 3);
        s.events.push(income("pay", 20_000, 2));
        s.goals.push(goal(25_000));
        assert_eq!(simulate(&s).goals[0].1, Affordability::FromMonth(2));
    }

    #[test]
    fn shortfall_is_measured_from_peak_balance() {
        let mut s = scenario(10_000, 0, 0, 3);
        s.events.push(income("pay", 20_000, 2));
        s.goals.push(goal(35_000));
        assert_eq!(
            simulate(&s).goals[0].1,
            Affordability::NotInHorizon { shortfall: 5_000 }
        );
    }

    #[test]
    fn reports_events_outside_horizon_and_no_runway_without_costs() {
        let mut s = scenario(10_000, 0, 0, 3);
        s.events.push(income("late", 1_000, 4));
        let report = simulate(&s);
        assert_eq!(report.skipped_events, vec!["late".to_string()]);
        assert_eq!(report.runway_months, None);
    }
}
