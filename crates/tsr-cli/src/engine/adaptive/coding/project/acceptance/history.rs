//! Explicit local episode carryover. Copy observations, never fitted weights or private text.
use super::*;
impl Engine {
    pub fn project_acceptance_carry(&self, source: &Engine) -> Result<Completion> {
        ensure!(
            self.directory.canonicalize()? != source.directory.canonicalize()?,
            "cannot import own episode"
        );
        source.store.connection().execute_batch("BEGIN")?;
        let snapshot = (|| -> Result<_> { Ok((source.records()?, source.store.head_root()?)) })();
        source.store.connection().execute_batch("ROLLBACK")?;
        let (source_records, source_root) = snapshot?;
        let (source_revision, source_requirements) = source_records
            .iter()
            .rev()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceRequirements { requirements },
                } => Some((*id, requirements)),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("source requirements missing"))?;
        let source_episode = source_records
            .first()
            .ok_or_else(|| anyhow::anyhow!("empty source episode"))?
            .0;
        self.write(|| {
            ensure!(self.adaptive_state_inner()?.pending.is_none(),"cannot change evidence during commitment");
            let records=self.records()?;
            let (revision,requirements)=records.iter().rev().find_map(|(id,r)|match r {
                Record::Coding {record:CodingRecord::AcceptanceRequirements {requirements}}=>Some((*id,requirements)),_=>None,
            }).ok_or_else(||anyhow::anyhow!("destination requirements missing"))?;
            ensure!(source_requirements.iter().all(|(id,text)|requirements.get(id)==Some(text)),"carry requires preserving source requirements; added requirements start without evidence");
            let mut observations=BTreeMap::new();
            let mut episodes=BTreeMap::new();
            for (_,r) in &source_records {
                if let Record::Coding {record:CodingRecord::AcceptanceHistory {revision,source_episode,observations,..}}=r {
                    if *revision==source_revision {episodes.insert(*source_episode,observations);}
                }
            }
            for points in episodes.values() {for point in *points {observations.insert(point.receipt,point.clone());}}
            for (id,r) in &source_records {
                match r {
                    Record::Coding {record:CodingRecord::AcceptanceReview {review}} if review.requirements_revision==source_revision=> {
                        if let Some(receipt)=review.test_receipt {
                            if review.decision==Decision::Unresolved {observations.remove(&receipt);}
                            else {
                                observations.insert(receipt,HistoricalReview {review:*id,receipt,accepted:review.decision==Decision::Accept,met:review.evidence.iter().map(|(id,e)|(id.clone(),e.met)).collect()});
                            }
                        }
                    }
                    _=>{}
                }
            }
            ensure!(observations.len()<=128,"carry at most 128 independently reviewed candidates per episode");
            let observations=observations.into_values().collect::<Vec<_>>();
            let already=records.iter().any(|(_,r)|matches!(r,Record::Coding {record:CodingRecord::AcceptanceHistory {revision:r,source_root:root,..}} if *r==revision && *root==source_root));
            if !already {
                self.adaptive_record(&Record::Coding {record:CodingRecord::AcceptanceHistory {revision,source_episode,source_root,observations}},Source::Operator)?;
            }
            self.project_acceptance_status()
        })
    }
}
