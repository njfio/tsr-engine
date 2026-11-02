//! Explicit origin joins are exports, never new edits, returns or training episodes.
use super::*;

fn immutable_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => file.write_all(bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ensure!(fs::read(path)? == bytes, "existing export artifact differs");
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

impl Engine {
    pub fn project_export_origin(
        &self,
        origin_path: &Path,
        commitment: EventId,
    ) -> Result<serde_json::Value> {
        ensure!(
            origin_path.join("events.sqlite").is_file(),
            "existing origin episode required"
        );
        let origin = Engine::open(origin_path)?;
        let origin_root = origin.store.head_root()?;
        let own_root = self.store.head_root()?;
        let spec = self.project_configuration()?;
        let origin_spec = origin.project_configuration()?;
        let current = self.project_snapshot(&spec)?;
        let candidate = digest(&current)?;
        let records = origin.records()?;
        let (intent_id, plan) = records
            .iter()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::Intent { plan },
                } if plan.commitment == commitment => Some((*id, plan.clone())),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("origin commitment has no executed intent"))?;
        ensure!(
            plan.generated && !plan.edits.is_empty(),
            "origin must be a generated edit, not verification"
        );
        ensure!(records.iter().any(|(_,r)| matches!(r,Record::Coding { record: CodingRecord::Receipt { commitment:c, source_after:Some(h), .. } } if *c == commitment && *h == candidate)), "origin lacks matching observed candidate return");
        ensure!(
            plan.expected_after == Some(candidate),
            "wrong origin candidate"
        );
        let mut initial = origin_spec.protected.clone();
        initial.extend(origin_spec.editable.clone());
        ensure!(
            digest(&initial)? == plan.source_before,
            "origin is not this initial admitted mutant"
        );
        ensure!(
            initial.keys().eq(current.keys()),
            "origin file scope differs"
        );
        let patches = origin
            .patches_for(&plan.action)?
            .ok_or_else(|| anyhow::anyhow!("origin generated proposal missing"))?;
        ensure!(
            !patches.is_empty() && patches.len() <= 4 && patches.len() == plan.edits.len(),
            "unsupported origin edit count"
        );
        let proposal_records: Vec<_> = records.iter().filter(|(_,r)| matches!(r,
            Record::Coding { record: CodingRecord::Patch { action, .. } | CodingRecord::PatchSet { action, .. } } if action.id == plan.action)).collect();
        let events = origin.events()?;
        ensure!(
            proposal_records.len() == 1
                && events
                    .iter()
                    .any(|(_, e)| e.id == proposal_records[0].0 && e.source == Source::Rules),
            "origin proposal lacks controller registration provenance"
        );
        let mut reconstructed = initial.clone();
        let mut paths = BTreeSet::new();
        for patch in &patches {
            ensure!(paths.insert(&patch.path), "duplicate origin path");
            let old = origin_spec
                .editable
                .get(&patch.path)
                .ok_or_else(|| anyhow::anyhow!("origin edit outside admitted paths"))?;
            ensure!(
                patch.source_digest == Hash::of(&[old.as_bytes()])
                    && !patch.old_text.is_empty()
                    && old.matches(&patch.old_text).count() == 1,
                "origin patch does not bind initial bytes"
            );
            let replacement = old.replacen(&patch.old_text, &patch.new_text, 1);
            ensure!(
                plan.edits
                    .iter()
                    .any(|e| e.path == patch.path && e.replacement == replacement),
                "origin intent differs from generated patch"
            );
            reconstructed.insert(patch.path.clone(), replacement);
        }
        ensure!(
            reconstructed == current && origin.project_snapshot(&origin_spec)? == current,
            "origin or candidate drift"
        );
        let completion = self.project_acceptance_status()?;
        ensure!(
            completion.accepted && completion.snapshot == candidate,
            "accepted current candidate required"
        );
        // This method never appends events or creates execution/resource requests.
        let mut diff = String::new();
        for (path, old) in &origin_spec.editable {
            let new = &current[path];
            if old == new {
                continue;
            }
            diff.push_str(&format!(
                "--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n",
                usize::from(!old.is_empty()),
                old.lines().count(),
                usize::from(!new.is_empty()),
                new.lines().count()
            ));
            for (prefix, text) in [("-", old), ("+", new)] {
                for line in text.split_inclusive('\n') {
                    diff.push_str(prefix);
                    diff.push_str(line);
                    if !line.ends_with('\n') {
                        diff.push_str("\n\\ No newline at end of file\n");
                    }
                }
            }
        }
        ensure!(!diff.is_empty(), "origin repair is empty");
        let proof = serde_json::json!({"version":"verified-repair-origin@v1","origin_root":origin_root,"origin_commitment":commitment,"origin_intent":intent_id,"origin_spec":origin_spec,"origin_plan":plan,"generated_patches":patches,"proposal_records":proposal_records,"initial_file_hashes":initial.iter().map(|(p,s)|(p.clone(),Hash::of(&[s.as_bytes()]))).collect::<BTreeMap<_,_>>(),"candidate_file_hashes":current.iter().map(|(p,s)|(p.clone(),Hash::of(&[s.as_bytes()]))).collect::<BTreeMap<_,_>>(),"initial_snapshot":plan.source_before,"candidate_snapshot":candidate,"candidate_files":current,"verification_root":own_root,"acceptance":completion,"repair_diff":diff,"repair_diff_hash":Hash::of(&[diff.as_bytes()]),"new_edit":false,"new_resource_charge":false,"training_episode":false});
        let bytes = serde_json::to_vec_pretty(&proof)?;
        let id = Hash::of(&[&bytes]);
        ensure!(
            origin.store.head_root()? == origin_root && self.store.head_root()? == own_root,
            "state changed during origin export"
        );
        ensure!(
            self.project_snapshot(&spec)? == current
                && origin.project_snapshot(&origin_spec)? == current,
            "source changed during export"
        );
        let dir = self.directory.join("exports");
        fs::create_dir_all(&dir)?;
        let proof_path = dir.join(format!("origin-{id}.json"));
        let diff_path = dir.join(format!("origin-{id}.patch"));
        immutable_file(&proof_path, &bytes)?;
        immutable_file(&diff_path, diff.as_bytes())?;
        Ok(
            serde_json::json!({"goal_accepted":true,"snapshot":candidate,"origin_proof":proof_path,"origin_proof_hash":id,"patch":diff_path,"new_edit":false,"new_resource_charge":false}),
        )
    }
}
