//! `tsr-forecast` — the named events of §B7, resolution of a forecast against a task's
//! returns, and scoring (Brier, log loss, reliability bins, the residual series).
//!
//! Nothing here is fitted. The calibrator that turns a named event into a probability is M4;
//! in M2 a forecast's `calibrated` stays `None` and prints as `uncalibrated`.
//!
//! The crate is a pure library over the domain contract: it reads [`tsr_domain::Event`]s and
//! [`tsr_domain::Forecast`]s and returns numbers. It stores nothing.

pub mod outcomes;
pub mod resolve;
pub mod score;

pub use outcomes::{
    common_decision_event_id, named_event_by_id, named_event_by_name, named_event_id, named_events,
    outcome_def_id, CensoringRule, InterventionRule, NamedEvent, RequiredEvidence,
    COMMON_DECISION_EVENT, COMMON_DECISION_EVENT_VERSION, INTUITION_STAGES,
};
pub use resolve::{
    event_id, forecaster_name, missing_evidence, resolve, Resolution, ResolvedForecast, TaskFacts,
};
pub use score::{
    bin_index, calibration_label, reliability_bins, residual_proxy, score_per_bundle,
    score_per_family, score_per_forecaster, BundleScore, FamilyScore, ForecasterScore,
    ReliabilityBin, ResidualPoint, ResidualSeries, Score, ScoredForecast, LOG_LOSS_EPSILON,
    RELIABILITY_BINS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{
        Event, EventKind, Forecast, ForecastId, Forecaster, Hash, Horizon, OutcomeDefId,
        Probability, Source, Support, TaskId,
    };

    fn task() -> TaskId {
        TaskId(Hash::of(&[b"task-1"]))
    }

    fn forecast(event: OutcomeDefId, source: Forecaster, p: f64) -> Forecast {
        Forecast {
            id: ForecastId(Hash::of(&[b"f"])),
            event,
            horizon: Horizon::TaskCloseDays(7),
            source,
            raw: Probability::new(p).unwrap(),
            calibrated: None,
            support: Support { episodes: 3 },
            state_version: 1,
        }
    }

    fn executed(exit_code: i32) -> Event {
        Event::new(
            task(),
            vec![],
            Source::Rules,
            EventKind::Executed {
                candidate: tsr_domain::CandidateId(Hash::of(&[b"c"])),
                exit_code: Some(exit_code),
                log_digest: None,
                duration_ms: 1,
            },
            "2026-10-11T00:00:00Z",
        )
    }

    fn closed(reason: &str) -> Event {
        Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::TaskClosed {
                task: task(),
                reason: reason.into(),
            },
            "2026-10-11T00:00:00Z",
        )
    }

    #[test]
    fn test_pass_resolves_true_on_exit_zero_and_false_otherwise() {
        let f = forecast(
            named_event_id("test_pass").unwrap(),
            Forecaster::Generator,
            0.8,
        );
        let facts = TaskFacts::from_events(&[executed(0)]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::True);
        let facts = TaskFacts::from_events(&[executed(1)]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::False);
    }

    #[test]
    fn test_pass_is_unresolved_without_an_execution_and_censored_on_cancellation() {
        let f = forecast(
            named_event_id("test_pass").unwrap(),
            Forecaster::Generator,
            0.8,
        );
        let facts = TaskFacts::from_events(&[]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Unresolved);
        let cancelled = Event::new(
            task(),
            vec![],
            Source::Rules,
            EventKind::Unresolved {
                candidate: tsr_domain::CandidateId(Hash::of(&[b"c"])),
                reason: "cancelled".into(),
            },
            "2026-10-11T00:00:00Z",
        );
        let facts = TaskFacts::from_events(&[cancelled]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Censored);
    }

    #[test]
    fn diff_accepted_reads_the_operator_label() {
        let f = forecast(
            named_event_id("diff_accepted").unwrap(),
            Forecaster::Operator,
            0.5,
        );
        let facts = TaskFacts::from_events(&[closed("operator")])
            .with_label("accepted", tsr_domain::EventId(Hash::of(&[b"l"])));
        assert_eq!(resolve(&f, &facts).resolution, Resolution::True);
        let facts = TaskFacts::from_events(&[closed("operator")])
            .with_label("rejected", tsr_domain::EventId(Hash::of(&[b"l"])));
        assert_eq!(resolve(&f, &facts).resolution, Resolution::False);
        let facts = TaskFacts::from_events(&[closed("operator")]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Censored);
    }

    #[test]
    fn common_event_needs_close_and_acceptance() {
        let f = forecast(common_decision_event_id(), Forecaster::Controller, 0.6);
        // Closed with no label: never reviewed, so censored (B7).
        let facts = TaskFacts::from_events(&[closed("done")]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Censored);
        // Closed done (criteria met) and accepted: the event happened.
        let facts = TaskFacts::from_events(&[closed("done")])
            .with_label("accepted", tsr_domain::EventId(Hash::of(&[b"l"])));
        assert_eq!(resolve(&f, &facts).resolution, Resolution::True);
        // Accepted but closed with reason `operator`: an operator close is an external
        // stop, so the common event is censored, not resolved false (R2).
        let facts = TaskFacts::from_events(&[closed("operator")])
            .with_label("accepted", tsr_domain::EventId(Hash::of(&[b"l"])));
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Censored);
        // Still open: unresolved.
        let facts = TaskFacts::from_events(&[]);
        assert_eq!(resolve(&f, &facts).resolution, Resolution::Unresolved);
    }

    #[test]
    fn brier_and_log_loss_on_hand_computed_cases() {
        let s = ScoredForecast {
            forecaster: Forecaster::Generator,
            p: 0.8,
            outcome: 1.0,
            family: None,
            bundle: None,
        };
        assert!((s.brier() - 0.04).abs() < 1e-12);
        assert!((s.log_loss() - -(0.8f64).ln()).abs() < 1e-12);
        let s = ScoredForecast {
            forecaster: Forecaster::Generator,
            p: 0.25,
            outcome: 0.0,
            family: None,
            bundle: None,
        };
        assert!((s.brier() - 0.0625).abs() < 1e-12);
        assert!((s.log_loss() - -(0.75f64).ln()).abs() < 1e-12);
        // A confident wrong forecast is finite, not infinite.
        let s = ScoredForecast {
            forecaster: Forecaster::Generator,
            p: 1.0,
            outcome: 0.0,
            family: None,
            bundle: None,
        };
        assert!(s.log_loss().is_finite());
    }

    #[test]
    fn score_means_over_a_known_sample() {
        let fs = vec![
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.8,
                outcome: 1.0,
                family: Some("parser".into()),
                bundle: None,
            },
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.2,
                outcome: 0.0,
                family: Some("parser".into()),
                bundle: None,
            },
        ];
        let s = Score::of(&fs);
        assert_eq!(s.n, 2);
        assert!((s.brier.unwrap() - 0.04).abs() < 1e-12);
        assert_eq!(s.bins.len(), RELIABILITY_BINS);
    }

    #[test]
    fn reliability_bins_count_a_known_sample() {
        let fs = vec![
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.05,
                outcome: 0.0,
                family: None,
                bundle: None,
            },
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.09,
                outcome: 1.0,
                family: None,
                bundle: None,
            },
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.95,
                outcome: 1.0,
                family: None,
                bundle: None,
            },
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 1.0,
                outcome: 1.0,
                family: None,
                bundle: None,
            },
        ];
        let bins = reliability_bins(&fs);
        assert_eq!(bins[0].count, 2, "0.05 and 0.09 fall in bin 0");
        assert_eq!(bins[9].count, 2, "0.95 and 1.0 fall in bin 9");
        assert_eq!(bins[0].mean_p, Some(0.07));
        assert_eq!(bins[0].observed, Some(0.5));
        assert_eq!(bins[5].count, 0);
        assert_eq!(bins[5].mean_p, None);
    }

    #[test]
    fn per_forecaster_family_and_bundle_grouping() {
        let fs = vec![
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.8,
                outcome: 1.0,
                family: Some("parser".into()),
                bundle: Some("b1".into()),
            },
            ScoredForecast {
                forecaster: Forecaster::Operator,
                p: 0.5,
                outcome: 0.0,
                family: Some("parser".into()),
                bundle: Some("b1".into()),
            },
            ScoredForecast {
                forecaster: Forecaster::Generator,
                p: 0.3,
                outcome: 0.0,
                family: Some("lexer".into()),
                bundle: Some("b2".into()),
            },
        ];
        let per_f = score_per_forecaster(&fs);
        assert_eq!(per_f.len(), 2);
        assert_eq!(per_f[0].name, "generator");
        assert_eq!(per_f[0].score.n, 2);
        assert_eq!(per_f[1].name, "operator");
        assert_eq!(per_f[1].score.n, 1);
        let per_fam = score_per_family(&fs);
        assert_eq!(per_fam.len(), 2);
        assert_eq!(per_fam[0].family, "parser");
        assert_eq!(per_fam[0].score.n, 2);
        let per_b = score_per_bundle(&fs);
        assert_eq!(per_b.len(), 2);
        assert_eq!(per_b[0].bundle, "b1");
    }

    #[test]
    fn residual_series_is_labeled_as_a_proxy_never_a_tsr_term() {
        let points = vec![
            (
                "parser".to_string(),
                ResidualPoint {
                    at: "2026-10-11T00:00:00Z".into(),
                    forecaster: Forecaster::Generator,
                    residual: 0.2,
                },
            ),
            (
                "parser".to_string(),
                ResidualPoint {
                    at: "2026-10-12T00:00:00Z".into(),
                    forecaster: Forecaster::Generator,
                    residual: -0.1,
                },
            ),
        ];
        let series = ResidualSeries::per_family(points);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].proxy, "expansion_dilation");
        assert_eq!(series[0].mean().unwrap().to_string(), "residual=0.05");
    }

    #[test]
    fn a_forecast_with_no_calibrator_prints_as_uncalibrated() {
        let f = forecast(common_decision_event_id(), Forecaster::Generator, 0.42);
        assert_eq!(calibration_label(&f), "uncalibrated");
        assert!(f.calibrated.is_none());
    }

    #[test]
    fn censored_and_unresolved_forecasts_are_never_scored() {
        let f = forecast(
            named_event_id("test_pass").unwrap(),
            Forecaster::Generator,
            0.8,
        );
        let facts = TaskFacts::from_events(&[]);
        let r = resolve(&f, &facts);
        assert!(ScoredForecast::from_resolved(&r, None, None).is_none());
        assert!(residual_proxy(&r).is_none());
    }
}
