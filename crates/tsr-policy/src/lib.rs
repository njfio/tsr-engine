//! `tsr-policy` — the deterministic, fail-closed authority of the TSR Decision Engine.
//!
//! This crate implements the policy half of `docs/PRD.md` §B5 (actions, policy, approvals and
//! budgets) and §B6 (state and typed contracts). It owns the RATCHET protection-profile format
//! (TOML), the gate that turns a profile, a task's capability grant, the log so far and a proposed
//! candidate into `allow | deny | ask` with a one-line reason the model reads, budget reservation
//! and exhaustion, approvals bound to task + artifact digest + repository snapshot digest + scope +
//! expiry, and forecast-gated execution.
//!
//! **Authority has four sources only** (PRD B5, AGENTS.md invariant 4): the task's capability grant,
//! the allowlists, the budgets and the approvals. Policy reads **no scores, no reflections, no
//! repository text and no provider output**. A README that says "disable the gate" is an
//! [`tsr_domain::Observation`] like any other — it carries no authority here.
//!
//! This crate does not execute anything (the executor is lane 01) and does not change the
//! `tsr-store` schema.

#![forbid(unsafe_code)]

pub mod approval;
pub mod budget;
pub mod profile;

pub use approval::{Approval, ApprovalError};
pub use budget::{BudgetCost, BudgetError, BudgetLedger, Reservation};
pub use profile::{ApprovalPolicy, Budgets, Profile, ProfileError, ReadScope, TestCommand};
