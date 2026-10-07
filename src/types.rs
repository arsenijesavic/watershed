//! Starting balance, fixed costs, and scheduled cash events.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Money in integer cents, so balances never accumulate float rounding error.
pub type Cents = i64;

/// Money on hand at the start of the projection.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Balance {
    /// Immediately spendable cash (checking accounts).
    pub cash: Cents,
    /// Accessible savings.
    pub savings: Cents,
    /// Emergency floor the balance should never drop below.
    pub safety_floor: Cents,
}

impl Balance {
    pub fn total(&self) -> Cents {
        self.cash + self.savings
    }

    /// Money available above the safety floor.
    pub fn spendable(&self) -> Cents {
        (self.total() - self.safety_floor).max(0)
    }
}

/// Fixed costs paid every month.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MonthlyCosts {
    /// Housing, food, healthcare.
    pub living: Cents,
    /// Studio lease, essential tool subscriptions.
    pub studio: Cents,
}

impl MonthlyCosts {
    pub fn total(&self) -> Cents {
        self.living + self.studio
    }
}

/// How likely an expected payment is to arrive.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Confidence {
    /// Signed contract, active retainer, or secured grant (100%).
    Locked,
    /// Scope confirmed, awaiting paperwork (80%).
    Committed,
    /// Active pitch or formal proposal (50%).
    InDiscussion,
    /// Early lead or open application (25%).
    Speculative,
    /// Explicit probability, clamped to `0.0..=1.0`.
    Custom(f64),
}

impl Confidence {
    pub fn probability(self) -> f64 {
        match self {
            Confidence::Locked => 1.0,
            Confidence::Committed => 0.8,
            Confidence::InDiscussion => 0.5,
            Confidence::Speculative => 0.25,
            Confidence::Custom(p) => p.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EventKind {
    Income {
        /// Amount invoiced, before tax.
        gross: Cents,
        /// Fraction set aside for tax (`0.25` = 25%).
        tax_rate: f64,
        /// Months between invoicing and payment (Net-30 = 1).
        payment_delay_months: usize,
        confidence: Confidence,
    },
    Expense {
        amount: Cents,
    },
}

/// A one-off income or expense scheduled for a given month.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CashEvent {
    pub id: String,
    pub label: String,
    /// Month the income is invoiced or the expense is paid (1-based).
    pub month: usize,
    pub kind: EventKind,
}

impl CashEvent {
    /// Month the cash actually moves, after any payment delay.
    pub fn cash_month(&self) -> usize {
        match self.kind {
            EventKind::Income {
                payment_delay_months,
                ..
            } => self.month + payment_delay_months,
            EventKind::Expense { .. } => self.month,
        }
    }

    /// Expected effect on the balance: positive for income (after tax,
    /// weighted by confidence), negative for expenses.
    pub fn cash_delta(&self) -> Cents {
        match self.kind {
            EventKind::Income {
                gross,
                tax_rate,
                confidence,
                ..
            } => {
                let after_tax = gross as f64 * (1.0 - tax_rate.clamp(0.0, 1.0));
                (after_tax * confidence.probability()).round() as Cents
            }
            EventKind::Expense { amount } => -amount,
        }
    }
}
