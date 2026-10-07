# Watershed

An ecological financial projection engine for creative practitioners.

*Moving from mechanical ledgers to carrying capacity, seasonal flow, and living soil.*

## Overview

Traditional financial tools treat personal money as an accounting ledger: rigid categories, backwards-looking receipts, and linear monthly burn. For independent creators, this framing creates friction and anxiety. Creative livelihoods are seasonal, non-linear, and defined by feast-and-drought cycles.

Watershed is a local-first projection engine that models finance as a living watershed:

- **The Basin & Bedrock** (`Balance`): your cash and savings, protected by a safety floor you never want to drop below.
- **Basal Evaporation** (`MonthlyCosts`): the fixed monthly cost of keeping your physical and creative life afloat.
- **Seasonal Weather** (`CashEvent`): one-off income and expenses. Income is weighted by how likely it is to arrive, delayed by client payment terms (Net-30/60), and reduced by tax set aside.
- **The Garden of Desires** (`Goal`): purchases and commitments (tools, residencies, releases, sabbaticals) that become affordable only when the watershed has enough carrying capacity.

All amounts are integer cents (`Cents = i64`), so balances never pick up floating-point rounding error.

## Core Concepts

### 1. The Water Table & Carrying Capacity

Runway is not calculated down to zero. Watershed measures runway down to your **safety floor**, the emergency reserve below which survival anxiety sets in:

```
runway_months = (cash + savings − safety_floor) / (living + studio)
```

If you have no fixed monthly costs, `runway_months` is `None`.

### 2. Latency & Confidence Dampening

A promised client invoice is not spendable cash until it lands. Each income event is reduced to its expected value:

```
expected_income = gross × (1 − tax_rate) × confidence
```

It is applied in month `month + payment_delay_months` (Net-30 = 1 month delay). Confidence levels:

| `Confidence`   | Probability | Meaning                                       |
|----------------|-------------|-----------------------------------------------|
| `Locked`       | 100%        | Signed contract, active retainer, secured grant |
| `Committed`    | 80%         | Scope confirmed, awaiting paperwork           |
| `InDiscussion` | 50%         | Active pitch or formal proposal               |
| `Speculative`  | 25%         | Early lead or open application                |
| `Custom(p)`    | `p`         | Explicit probability, clamped to 0–1          |

Events that land outside the projected months are not applied and are listed in `report.skipped_events`.

### 3. The Garden of Desires

Instead of forcing a binary "yes or no" on creative purchases, each goal needs the balance to reach:

```
required = safety_floor + safety_buffer + capital_needed
```

`capital_needed` is the price of a `GoalCost::OneTime`, or `monthly × buffer_months` for a `GoalCost::Recurring` commitment. Each goal gets one of three states:

- **In Bloom** (`Affordability::Now`): the opening balance already covers it.
- **Germinating** (`Affordability::FromMonth(m)`): projected income lifts the balance to the required level in month `m`.
- **Dormant** (`Affordability::NotInHorizon { shortfall }`): the balance never gets there. `shortfall` is how far the highest projected balance falls short.

Each goal is evaluated on its own. Taking one on does not reduce what is available for the others.

## Architecture

The engine is a pure, deterministic function, `simulate(&Scenario) -> Report`, with no required dependencies (`serde` is optional).

```
INPUT (Scenario)                ENGINE (simulate)                  OUTPUT (Report)
┌────────────────────────┐      ┌──────────────────────────┐      ┌──────────────────────────────┐
│ • balance              │      │ • Tax set-aside          │      │ • runway_months              │
│ • costs (living/studio)│─────>│ • Payment delay shift    │─────>│ • timeline (month by month)  │
│ • events               │      │ • Confidence weighting   │      │ • low_point, first_breach    │
│ • goals                │      │ • Monthly timeline step  │      │ • goals (affordability)      │
│ • months               │      │                          │      │ • skipped_events             │
└────────────────────────┘      └──────────────────────────┘      └──────────────────────────────┘
```

| Module     | Contents                                                            |
|------------|---------------------------------------------------------------------|
| `types`    | `Cents`, `Balance`, `MonthlyCosts`, `Confidence`, `EventKind`, `CashEvent` |
| `goals`    | `Goal`, `GoalCost`, `Affordability`                                 |
| `engine`   | `Scenario`, `MonthRecord`, `Report`, `simulate`                     |

## Quickstart

### Prerequisites

- A recent stable Rust toolchain with Cargo

### Installation

```bash
git clone https://github.com/arsenijesavic/watershed.git
cd watershed
```

### Running the Example Simulation

```bash
cargo run --example simulate
```

### Running the Tests

```bash
cargo test
```

### Optional serde Support

Enable the `serde` feature to derive `Serialize`/`Deserialize` on every public type:

```toml
watershed = { version = "0.1", features = ["serde"] }
```

## Usage Example

```rust
use watershed::{
    simulate, Balance, CashEvent, Confidence, EventKind, Goal, GoalCost, MonthlyCosts, Scenario,
};

fn main() {
    let scenario = Scenario {
        balance: Balance {
            cash: 2_000_000,         // $20,000.00 (all amounts are cents)
            savings: 600_000,        // $6,000.00
            safety_floor: 600_000,   // Untouchable safety reserve
        },
        costs: MonthlyCosts {
            living: 420_000,
            studio: 80_000,          // $5,000 monthly burn
        },
        events: vec![CashEvent {
            id: "contract_alpha".into(),
            label: "Client Commission".into(),
            month: 2,
            kind: EventKind::Income {
                gross: 4_000_000,
                tax_rate: 0.25,
                payment_delay_months: 1, // Lands in month 3 (Net-30)
                confidence: Confidence::Locked,
            },
        }],
        goals: vec![Goal {
            id: "rig".into(),
            label: "Studio Hardware Upgrade".into(),
            cost: GoalCost::OneTime(400_000),
            safety_buffer: 500_000,
        }],
        months: 6,
    };

    let report = simulate(&scenario);

    if let Some(runway) = report.runway_months {
        println!("Runway: {runway:.1} months");
    }
    println!(
        "Low point: {} cents (month {})",
        report.low_point, report.low_point_month
    );

    for (goal_id, affordability) in &report.goals {
        println!("Goal [{goal_id}]: {affordability:?}");
    }
}
```

## Project Status

- [x] Pure deterministic simulation engine in Rust
- [x] Confidence weighting and tax set-aside
- [x] Payment delay time-shifting
- [x] Goal affordability evaluation
- [x] Optional `serde` serialization
- [ ] Goals that draw from a shared balance and affect each other
- [ ] Local-first GUI / Visual Canvas (Svelte + Tauri / WebAssembly)
- [ ] Exportable event logs (JSON / SQLite)

## Philosophy

Watershed is built on living systems principles:

- **Time sovereignty over profit optimization:** the primary asset of an independent creator is uninterrupted creative focus.
- **Ecological realism:** modeling irregular cash flows honestly prevents fear-based project selection.
- **Coherence and care:** tools should cultivate calm awareness of one's seasons rather than administrative guilt.

## License

Dual-licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT License](LICENSE-MIT), at your option.
