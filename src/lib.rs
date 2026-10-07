//! # Watershed
//!
//! A cash-flow projection engine for creative practitioners.
//!
//! Think of your finances as a watershed: a reservoir of savings with a safety
//! floor, a steady monthly drain, and rain that arrives late and uncertain.
//! Watershed projects that balance month by month and tells you when the goals
//! you care about become affordable.

pub mod engine;
pub mod goals;
pub mod types;

pub use engine::{simulate, MonthRecord, Report, Scenario};
pub use goals::{Affordability, Goal, GoalCost};
pub use types::{Balance, CashEvent, Cents, Confidence, EventKind, MonthlyCosts};
