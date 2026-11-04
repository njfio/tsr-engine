//! The reading view (spec #10): a fold over the log, nothing stored. A reading's want crosses when
//! it is recorded, with no reviewer. It stays out of the preference fold, because open preferences
//! feed refinement parents and acceptance aims. Returns sit beside readings with no verdict.
use crate::{normalize_text, Ledger, LedgerError};
use serde::{Deserialize, Serialize};
use tsr_domain::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentWant {
    pub text: String,
    pub reading: EventId,
    pub origin: Source,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReturnEntry {
    pub event: EventId,
    pub from: Source,
    pub text: String,
    pub evidence: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingEntry {
    pub event: EventId,
    pub reading: Reading,
    pub returns: Vec<ReturnEntry>,
}

/// Counts only, with no claim about what they mean.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingCounts {
    pub readings: u64,
    pub blank: u64,
    pub restated: u64,
    pub changed_after_return: u64,
    pub returns_named: u64,
    pub returns_unnamed: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingView {
    pub current_want: Option<CurrentWant>,
    pub readings: Vec<ReadingEntry>,
    pub unattached_returns: Vec<ReturnEntry>,
    pub counts: ReadingCounts,
}

/// What a request carries. Everything else stays in the log, and the omitted counts say so.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedReadings {
    pub current_want: Option<CurrentWant>,
    pub readings: Vec<ReadingEntry>,
    pub omitted: u64,
    pub unattached_returns: Vec<ReturnEntry>,
    pub omitted_returns: u64,
}

/// Only `Operator` and `Rules` can return. The generator never writes its own return.
fn may_return(from: Source) -> bool {
    matches!(from, Source::Operator | Source::Rules)
}

/// The whole reading record for one task, read from the log. An event that fails the write checks
/// is ignored, so a forged or malformed event opens nothing.
pub fn reading_view(events: &[(i64, Event)], task: &TaskId) -> ReadingView {
    let mut view = ReadingView::default();
    let mut return_since_reading = false;
    for (_, ev) in events.iter().filter(|(_, ev)| ev.task == *task) {
        match &ev.kind {
            EventKind::Reading { reading } => {
                if ev.source != Source::Rules || reading.validate().is_err() {
                    continue;
                }
                let earlier = view.readings.last().and_then(|e| e.reading.want.as_deref());
                if let (Some(earlier), Some(want)) = (earlier, reading.want.as_deref()) {
                    if normalize_text(earlier) == normalize_text(want) {
                        view.counts.restated += 1;
                    } else if return_since_reading {
                        view.counts.changed_after_return += 1;
                    }
                }
                view.counts.readings += 1;
                view.counts.blank += u64::from(reading.is_blank());
                view.current_want = reading.want.as_ref().map(|text| CurrentWant {
                    text: text.clone(),
                    reading: ev.id,
                    origin: Source::Generator,
                });
                view.readings.push(ReadingEntry {
                    event: ev.id,
                    reading: reading.clone(),
                    returns: vec![],
                });
                return_since_reading = false;
            }
            EventKind::Returned { returned } => {
                if !may_return(ev.source) || returned.validate().is_err() {
                    continue;
                }
                let entry = ReturnEntry {
                    event: ev.id,
                    from: ev.source,
                    text: returned.text.clone(),
                    evidence: returned.evidence,
                };
                match returned.reading {
                    None => {
                        view.counts.returns_unnamed += 1;
                        view.unattached_returns.push(entry);
                    }
                    Some(id) => {
                        let Some(named) = view.readings.iter_mut().find(|e| e.event == id) else {
                            continue;
                        };
                        view.counts.returns_named += 1;
                        named.returns.push(entry);
                    }
                }
                return_since_reading = true;
            }
            _ => {}
        }
    }
    view
}

fn latest<T: Clone>(items: &[T], limit: usize) -> (Vec<T>, u64) {
    let keep = items.len().min(limit);
    (
        items[items.len() - keep..].to_vec(),
        (items.len() - keep) as u64,
    )
}

/// The latest `limit` readings and returns, with what was left out counted.
pub fn bounded_readings(view: &ReadingView, limit: usize) -> BoundedReadings {
    let (mut readings, omitted) = latest(&view.readings, limit);
    let (unattached_returns, mut omitted_returns) = latest(&view.unattached_returns, limit);
    for entry in &mut readings {
        let (returns, left_out) = latest(&entry.returns, limit);
        entry.returns = returns;
        omitted_returns += left_out;
    }
    BoundedReadings {
        current_want: view.current_want.clone(),
        readings,
        omitted,
        unattached_returns,
        omitted_returns,
    }
}

impl Ledger<'_> {
    pub fn record_reading(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        reading: Reading,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        reading.validate()?;
        let event = Event::new(
            *task,
            parents,
            Source::Rules,
            EventKind::Reading { reading },
            at,
        );
        self.store().append(&event)?;
        Ok(event.id)
    }

    pub fn record_return(
        &self,
        task: &TaskId,
        from: Source,
        returned: ReadingReturn,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        if !may_return(from) {
            return Err(LedgerError::InvalidReview(
                "a return comes from the operator or rules",
            ));
        }
        returned.validate()?;
        if let Some(id) = returned.reading {
            let view = reading_view(&self.store().all()?, task);
            if !view.readings.iter().any(|e| e.event == id) {
                return Err(LedgerError::InvalidReview(
                    "a return names a reading in this task",
                ));
            }
        }
        let event = Event::new(*task, vec![], from, EventKind::Returned { returned }, at);
        self.store().append(&event)?;
        Ok(event.id)
    }
}
