use watershed::{
    simulate, Affordability, Balance, CashEvent, Cents, Confidence, EventKind, Goal, GoalCost,
    MonthlyCosts, Scenario,
};

fn dollars(amount: i64) -> Cents {
    amount * 100
}

fn fmt(cents: Cents) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}${}.{:02}", cents.abs() / 100, cents.abs() % 100)
}

fn main() {
    let scenario = Scenario {
        balance: Balance {
            cash: dollars(20_000),
            savings: dollars(6_000),
            safety_floor: dollars(6_000),
        },
        costs: MonthlyCosts {
            living: dollars(4_200),
            studio: dollars(800),
        },
        events: vec![
            CashEvent {
                id: "commission_alpha".into(),
                label: "Album Production Commission".into(),
                month: 2,
                kind: EventKind::Income {
                    gross: dollars(40_000),
                    tax_rate: 0.25,
                    payment_delay_months: 1,
                    confidence: Confidence::Locked,
                },
            },
            CashEvent {
                id: "arts_grant".into(),
                label: "Creative Sabbatical Grant Proposal".into(),
                month: 5,
                kind: EventKind::Income {
                    gross: dollars(20_000),
                    tax_rate: 0.15,
                    payment_delay_months: 0,
                    confidence: Confidence::Speculative,
                },
            },
        ],
        goals: vec![
            Goal {
                id: "synth".into(),
                label: "Modular Synthesizer".into(),
                cost: GoalCost::OneTime(dollars(4_000)),
                safety_buffer: dollars(5_000),
            },
            Goal {
                id: "residency".into(),
                label: "Self-Directed Residency".into(),
                cost: GoalCost::OneTime(dollars(12_000)),
                safety_buffer: dollars(8_000),
            },
        ],
        months: 6,
    };

    let report = simulate(&scenario);

    println!("=== SUMMARY ===");
    match report.runway_months {
        Some(months) => println!("Runway with no income: {months:.2} months"),
        None => println!("Runway with no income: unlimited (no fixed costs)"),
    }
    println!(
        "Low point: {} (month {})",
        fmt(report.low_point),
        report.low_point_month
    );
    match report.first_breach_month {
        Some(month) => println!("Safety floor breached in month {month}"),
        None => println!("Safety floor never breached"),
    }
    if !report.skipped_events.is_empty() {
        println!(
            "Outside projection, not applied: {}",
            report.skipped_events.join(", ")
        );
    }

    println!("\n=== MONTHLY BALANCE ===");
    for r in &report.timeline {
        println!(
            "Month {:02} | Open {:>10} | In +{:>10} | Out -{:>10} | Close {:>10}",
            r.month,
            fmt(r.opening),
            fmt(r.income),
            fmt(r.expenses),
            fmt(r.closing)
        );
    }

    println!("\n=== GOALS ===");
    for (id, affordability) in &report.goals {
        match affordability {
            Affordability::Now => println!("  {id}: affordable now"),
            Affordability::FromMonth(month) => println!("  {id}: affordable from month {month}"),
            Affordability::NotInHorizon { shortfall } => {
                println!("  {id}: not within projection (short {})", fmt(*shortfall))
            }
        }
    }
}
