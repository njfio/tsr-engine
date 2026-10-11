//! Budgets, reserved before dispatch (PRD §B5).
//!
//! Per task: generator tokens, tool minutes, wall clock, generator calls, patch bytes. A
//! reservation is taken **before** dispatch; on exhaustion the task stops with `Stop(budget)` and
//! marks any effect in flight as unresolved until reconciled.
//!
//! The ledger tracks two numbers per budget: **reserved** (held by in-flight reservations) and
//! **consumed** (committed). A reservation is refused when `reserved + consumed + cost` would
//! exceed the ceiling, so two in-flight reservations can never jointly overrun a budget. A
//! reservation that is [`BudgetLedger::release`]d frees its hold; one that is
//! [`BudgetLedger::commit`]ed moves its hold into consumed.
//!
//! The ledger is a pure, deterministic fold over the reservations it has seen: it reads no scores,
//! no reflections, no repository text and no provider output.

use tsr_domain::ActionKind;

use crate::profile::Budgets;

/// The cost a single dispatch would draw from each budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BudgetCost {
    /// Generator tokens.
    pub generator_tokens: u64,
    /// Tool minutes.
    pub tool_minutes: u32,
    /// Wall-clock seconds.
    pub wall_clock_secs: u64,
    /// Generator calls.
    pub generator_calls: u32,
    /// Patch bytes.
    pub patch_bytes: u64,
}

impl BudgetCost {
    /// A zero-cost reservation (e.g. a `Stop` action).
    pub const ZERO: BudgetCost = BudgetCost {
        generator_tokens: 0,
        tool_minutes: 0,
        wall_clock_secs: 0,
        generator_calls: 0,
        patch_bytes: 0,
    };
}

/// A held reservation. It names the action it was taken for and the cost it holds, so it can be
/// committed or released exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reservation {
    /// The action the reservation was taken for.
    pub kind: ActionKind,
    /// The cost held.
    pub cost: BudgetCost,
}

/// Why a reservation was refused.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BudgetError {
    /// The reservation would exceed the named budget's ceiling.
    #[error("budget exhausted: {budget}")]
    Exhausted {
        /// The name of the exhausted budget (`generator_tokens`, `tool_minutes`,
        /// `wall_clock_secs`, `generator_calls`, `patch_bytes`).
        budget: &'static str,
    },
}

/// The five budget names, in the order [`BudgetLedger::exhausted`] reports them.
const BUDGET_NAMES: [&str; 5] = [
    "generator_tokens",
    "tool_minutes",
    "wall_clock_secs",
    "generator_calls",
    "patch_bytes",
];

/// Reserved-vs-consumed tracking for all five budgets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetLedger {
    ceilings: Budgets,
    reserved: BudgetCost,
    consumed: BudgetCost,
}

impl BudgetLedger {
    /// A fresh ledger with nothing reserved and nothing consumed.
    pub fn new(ceilings: Budgets) -> Self {
        Self {
            ceilings,
            reserved: BudgetCost::ZERO,
            consumed: BudgetCost::ZERO,
        }
    }

    /// The ceilings this ledger enforces.
    pub fn ceilings(&self) -> &Budgets {
        &self.ceilings
    }

    /// The cost currently held by in-flight reservations.
    pub fn reserved(&self) -> &BudgetCost {
        &self.reserved
    }

    /// The cost committed so far.
    pub fn consumed(&self) -> &BudgetCost {
        &self.consumed
    }

    /// Reserve `cost` for `kind` **before dispatch**. Returns `Err` naming the first budget whose
    /// ceiling would be exceeded by `reserved + consumed + cost`.
    pub fn reserve(
        &mut self,
        kind: ActionKind,
        cost: &BudgetCost,
    ) -> Result<Reservation, BudgetError> {
        if let Some(budget) = self.first_overrun(cost) {
            return Err(BudgetError::Exhausted { budget });
        }
        self.reserved.generator_tokens += cost.generator_tokens;
        self.reserved.tool_minutes += cost.tool_minutes;
        self.reserved.wall_clock_secs += cost.wall_clock_secs;
        self.reserved.generator_calls += cost.generator_calls;
        self.reserved.patch_bytes += cost.patch_bytes;
        Ok(Reservation { kind, cost: *cost })
    }

    /// Commit a reservation: its hold becomes consumed. The reservation is spent.
    pub fn commit(&mut self, r: Reservation) {
        self.reserved.generator_tokens -= r.cost.generator_tokens;
        self.reserved.tool_minutes -= r.cost.tool_minutes;
        self.reserved.wall_clock_secs -= r.cost.wall_clock_secs;
        self.reserved.generator_calls -= r.cost.generator_calls;
        self.reserved.patch_bytes -= r.cost.patch_bytes;

        self.consumed.generator_tokens += r.cost.generator_tokens;
        self.consumed.tool_minutes += r.cost.tool_minutes;
        self.consumed.wall_clock_secs += r.cost.wall_clock_secs;
        self.consumed.generator_calls += r.cost.generator_calls;
        self.consumed.patch_bytes += r.cost.patch_bytes;
    }

    /// Release a reservation: its hold is freed and nothing is consumed.
    pub fn release(&mut self, r: Reservation) {
        self.reserved.generator_tokens -= r.cost.generator_tokens;
        self.reserved.tool_minutes -= r.cost.tool_minutes;
        self.reserved.wall_clock_secs -= r.cost.wall_clock_secs;
        self.reserved.generator_calls -= r.cost.generator_calls;
        self.reserved.patch_bytes -= r.cost.patch_bytes;
    }

    /// The first budget that is fully spent (`reserved + consumed == ceiling`), if any. A budget
    /// with headroom returns `None` for that slot.
    pub fn exhausted(&self) -> Option<&'static str> {
        let checks: [(bool, &'static str); 5] = [
            (
                self.reserved.generator_tokens + self.consumed.generator_tokens
                    >= self.ceilings.generator_tokens,
                BUDGET_NAMES[0],
            ),
            (
                self.reserved.tool_minutes + self.consumed.tool_minutes
                    >= self.ceilings.tool_minutes,
                BUDGET_NAMES[1],
            ),
            (
                self.reserved.wall_clock_secs + self.consumed.wall_clock_secs
                    >= self.ceilings.wall_clock_secs,
                BUDGET_NAMES[2],
            ),
            (
                self.reserved.generator_calls + self.consumed.generator_calls
                    >= self.ceilings.generator_calls,
                BUDGET_NAMES[3],
            ),
            (
                self.reserved.patch_bytes + self.consumed.patch_bytes >= self.ceilings.patch_bytes,
                BUDGET_NAMES[4],
            ),
        ];
        checks
            .iter()
            .find(|(spent, _)| *spent)
            .map(|(_, name)| *name)
    }

    /// True when no further dispatch may be reserved: some budget is exhausted.
    pub fn blocks_dispatch(&self) -> bool {
        self.exhausted().is_some()
    }

    /// The first budget that `cost` would overrun, if any.
    fn first_overrun(&self, cost: &BudgetCost) -> Option<&'static str> {
        let checks: [(bool, &'static str); 5] = [
            (
                self.reserved.generator_tokens
                    + self.consumed.generator_tokens
                    + cost.generator_tokens
                    > self.ceilings.generator_tokens,
                BUDGET_NAMES[0],
            ),
            (
                self.reserved.tool_minutes + self.consumed.tool_minutes + cost.tool_minutes
                    > self.ceilings.tool_minutes,
                BUDGET_NAMES[1],
            ),
            (
                self.reserved.wall_clock_secs
                    + self.consumed.wall_clock_secs
                    + cost.wall_clock_secs
                    > self.ceilings.wall_clock_secs,
                BUDGET_NAMES[2],
            ),
            (
                self.reserved.generator_calls
                    + self.consumed.generator_calls
                    + cost.generator_calls
                    > self.ceilings.generator_calls,
                BUDGET_NAMES[3],
            ),
            (
                self.reserved.patch_bytes + self.consumed.patch_bytes + cost.patch_bytes
                    > self.ceilings.patch_bytes,
                BUDGET_NAMES[4],
            ),
        ];
        checks.iter().find(|(over, _)| *over).map(|(_, name)| *name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ceilings() -> Budgets {
        Budgets {
            generator_tokens: 1000,
            tool_minutes: 10,
            wall_clock_secs: 600,
            generator_calls: 4,
            patch_bytes: 4096,
        }
    }

    fn cost() -> BudgetCost {
        BudgetCost {
            generator_tokens: 100,
            tool_minutes: 1,
            wall_clock_secs: 60,
            generator_calls: 1,
            patch_bytes: 512,
        }
    }

    #[test]
    fn reservation_is_taken_before_dispatch() {
        let mut ledger = BudgetLedger::new(ceilings());
        assert_eq!(ledger.reserved(), &BudgetCost::ZERO);
        assert_eq!(ledger.consumed(), &BudgetCost::ZERO);

        let r = ledger
            .reserve(ActionKind::Generate, &cost())
            .expect("first reservation fits");
        assert_eq!(r.kind, ActionKind::Generate);
        // The hold is visible before anything is committed.
        assert_eq!(ledger.reserved().generator_tokens, 100);
        assert_eq!(ledger.consumed().generator_tokens, 0);
        assert!(!ledger.blocks_dispatch());
    }

    #[test]
    fn commit_moves_the_hold_into_consumed() {
        let mut ledger = BudgetLedger::new(ceilings());
        let r = ledger.reserve(ActionKind::Generate, &cost()).unwrap();
        ledger.commit(r);
        assert_eq!(ledger.reserved(), &BudgetCost::ZERO);
        assert_eq!(ledger.consumed().generator_tokens, 100);
        assert_eq!(ledger.consumed().generator_calls, 1);
    }

    #[test]
    fn release_frees_the_budget() {
        let mut ledger = BudgetLedger::new(ceilings());
        let r = ledger.reserve(ActionKind::Generate, &cost()).unwrap();
        ledger.release(r);
        assert_eq!(ledger.reserved(), &BudgetCost::ZERO);
        assert_eq!(ledger.consumed(), &BudgetCost::ZERO);
        // The freed budget can be reserved again.
        assert!(ledger.reserve(ActionKind::Generate, &cost()).is_ok());
    }

    #[test]
    fn exhaustion_blocks_the_next_dispatch() {
        let mut ledger = BudgetLedger::new(ceilings());
        // Four generator calls is the ceiling.
        for _ in 0..4 {
            let r = ledger.reserve(ActionKind::Generate, &cost()).unwrap();
            ledger.commit(r);
        }
        assert_eq!(ledger.exhausted(), Some("generator_calls"));
        assert!(ledger.blocks_dispatch());
        // The next dispatch is refused, and the error names the exhausted budget.
        assert_eq!(
            ledger.reserve(ActionKind::Generate, &cost()),
            Err(BudgetError::Exhausted {
                budget: "generator_calls"
            })
        );
    }

    #[test]
    fn two_in_flight_reservations_cannot_jointly_overrun() {
        let mut ledger = BudgetLedger::new(ceilings());
        // Hold 4 calls without committing: the ceiling is reached by holds alone.
        let held: Vec<Reservation> = (0..4)
            .map(|_| ledger.reserve(ActionKind::Generate, &cost()).unwrap())
            .collect();
        assert_eq!(ledger.exhausted(), Some("generator_calls"));
        assert!(ledger.blocks_dispatch());
        assert!(ledger.reserve(ActionKind::Generate, &cost()).is_err());
        // Releasing one hold frees exactly one slot.
        ledger.release(held[0]);
        assert!(ledger.reserve(ActionKind::Generate, &cost()).is_ok());
    }

    #[test]
    fn the_error_names_the_first_exhausted_budget() {
        let mut ledger = BudgetLedger::new(ceilings());
        // A cost that overruns tokens and calls at once names tokens (the first checked).
        let big = BudgetCost {
            generator_tokens: 10_000,
            generator_calls: 100,
            ..BudgetCost::ZERO
        };
        assert_eq!(
            ledger.reserve(ActionKind::Generate, &big),
            Err(BudgetError::Exhausted {
                budget: "generator_tokens"
            })
        );
    }

    #[test]
    fn a_zero_cost_reservation_always_fits() {
        let mut ledger = BudgetLedger::new(ceilings());
        for _ in 0..100 {
            let r = ledger
                .reserve(ActionKind::Stop, &BudgetCost::ZERO)
                .expect("zero cost never overruns");
            ledger.commit(r);
        }
        assert!(!ledger.blocks_dispatch());
    }
}
