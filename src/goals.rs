//! Purchases and commitments you want to afford, and when you can.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::types::Cents;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GoalCost {
    /// Single purchase (instrument, camera body, print run).
    OneTime(Cents),
    /// Ongoing monthly cost; `buffer_months` of it must be in hand before starting.
    Recurring {
        monthly: Cents,
        buffer_months: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Goal {
    pub id: String,
    pub label: String,
    pub cost: GoalCost,
    /// Extra cushion required above the safety floor.
    pub safety_buffer: Cents,
}

impl Goal {
    /// Capital needed to take this on, excluding the floor and buffer.
    pub fn capital_needed(&self) -> Cents {
        match self.cost {
            GoalCost::OneTime(amount) => amount,
            GoalCost::Recurring {
                monthly,
                buffer_months,
            } => monthly * buffer_months as Cents,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Affordability {
    /// The opening balance already covers it.
    Now,
    /// The projected balance first reaches the required level in this month.
    FromMonth(usize),
    /// Never reached within the projection; `shortfall` is measured from the peak balance.
    NotInHorizon { shortfall: Cents },
}
