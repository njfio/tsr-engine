//! One reading each cycle (spec #10). The want crosses with no reviewer, and nothing here reaches
//! acceptance, scope or execution: readings live in their own view, outside the preference fold.
use super::*;
use serde_json::{json, Value};
use tsr_domain::{Reading, ReadingReturn};
use tsr_ledger::{bounded_readings, reading_view};

/// Readings and returns a request carries. The rest stays in the log and is counted as omitted.
pub const READING_REQUEST_BOUND: usize = 8;

impl Engine {
    fn reading_timestamp() -> Result<String> {
        Ok(format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        ))
    }

    /// Records a reading, with its text kept as generated and the controller as the writer.
    pub fn project_reading_record(&self, reading: Reading) -> Result<Value> {
        self.write(|| {
            let parents = self
                .events()?
                .last()
                .map(|(_, e)| vec![e.id])
                .unwrap_or_default();
            let id = Ledger::new(&self.store).record_reading(
                &Self::task(),
                parents,
                reading,
                Self::reading_timestamp()?,
            )?;
            Ok(json!({"reading": id}))
        })
    }

    /// Records a return from the operator, or from a controller-recorded check.
    pub fn project_return_record(&self, from: Source, returned: ReadingReturn) -> Result<Value> {
        self.write(|| {
            let id = Ledger::new(&self.store).record_return(
                &Self::task(),
                from,
                returned,
                Self::reading_timestamp()?,
            )?;
            Ok(json!({"returned": id}))
        })
    }

    /// What the next request shows, and the run's counts. No verdict is computed.
    pub fn project_reading_view(&self) -> Result<Value> {
        let view = reading_view(&self.events()?, &Self::task());
        Ok(json!({
            "bounded": bounded_readings(&view, READING_REQUEST_BOUND),
            "counts": view.counts,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(browser::tests::spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "orientation".into(),
            "Keep navigation understandable".into(),
        )]))?;
        Ok((dir, e))
    }

    fn wanting(want: &str) -> Reading {
        Reading {
            want: Some(want.into()),
            sense: None,
            belief: None,
            expect: None,
            expected_return: Some(tsr_domain::ExpectedReturn {
                from: "the operator".into(),
                what: "says it is harder".into(),
            }),
            falsifier: Some("the operator says it is too easy".into()),
            gap: None,
        }
    }

    #[test]
    fn spec_c10_want_changes_no_aim_or_preference() -> Result<()> {
        let (_d, e) = setup()?;
        let aims = e.acceptance_active_aims()?;
        let entries = Ledger::new(&e.store).collective()?.entries;
        e.project_reading_record(wanting("a harder second level"))?;
        assert_eq!(e.acceptance_active_aims()?, aims);
        assert_eq!(Ledger::new(&e.store).collective()?.entries, entries);
        Ok(())
    }

    #[test]
    fn spec_c10_want_leaves_the_cycle_branch_unchanged() -> Result<()> {
        let (_d, e) = setup()?;
        let before = e.project_cycle_request()?["branch"].clone();
        e.project_reading_record(wanting("a harder second level"))?;
        assert_eq!(e.project_cycle_request()?["branch"], before);
        Ok(())
    }

    #[test]
    fn spec_c07_view_shows_a_return_beside_its_reading() -> Result<()> {
        let (_d, e) = setup()?;
        let recorded = e.project_reading_record(wanting("a harder second level"))?;
        let reading = serde_json::from_value(recorded["reading"].clone())?;
        e.project_return_record(
            Source::Operator,
            ReadingReturn {
                reading: Some(reading),
                text: "waaaay too easy".into(),
                evidence: None,
            },
        )?;
        let view = e.project_reading_view()?;
        assert_eq!(
            view["bounded"]["readings"][0]["returns"][0]["text"],
            "waaaay too easy"
        );
        assert_eq!(
            view["bounded"]["readings"][0]["returns"][0]["from"],
            "operator"
        );
        assert_eq!(view["counts"]["returns_named"], 1);
        Ok(())
    }

    #[test]
    fn spec_c07_operator_return_leaves_the_project_readable() -> Result<()> {
        let (_d, e) = setup()?;
        e.project_return_record(
            Source::Operator,
            ReadingReturn {
                reading: None,
                text: "waaaay too easy".into(),
                evidence: None,
            },
        )?;
        e.project_configuration()?;
        e.project_cycle_request()?;
        Ok(())
    }

    #[test]
    fn reading_and_return_record_the_wall_clock_as_provenance() -> Result<()> {
        let millis = || -> Result<u128> {
            Ok(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis())
        };
        let (_d, e) = setup()?;
        let before = millis()?;
        e.project_reading_record(wanting("a harder second level"))?;
        e.project_return_record(
            Source::Operator,
            ReadingReturn {
                reading: None,
                text: "waaaay too easy".into(),
                evidence: None,
            },
        )?;
        let after = millis()?;
        let events = e.events()?;
        for (_, event) in events.iter().rev().take(2) {
            let at: u128 = event
                .recorded_at
                .strip_prefix("unix-ms:")
                .ok_or_else(|| anyhow::anyhow!("no unix-ms prefix"))?
                .parse()?;
            assert!(
                (before..=after).contains(&at),
                "{at} not in {before}..={after}"
            );
        }
        Ok(())
    }

    #[test]
    fn spec_c07_generated_return_is_refused() -> Result<()> {
        let (_d, e) = setup()?;
        let bad = ReadingReturn {
            reading: None,
            text: "I did well".into(),
            evidence: None,
        };
        assert!(e.project_return_record(Source::Generator, bad).is_err());
        assert_eq!(e.project_reading_view()?["counts"]["returns_unnamed"], 0);
        Ok(())
    }
}
