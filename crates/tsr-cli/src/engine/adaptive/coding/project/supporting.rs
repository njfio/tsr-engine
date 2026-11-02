//! Bounded owner-admitted observations, separate from editable source and patch handles.
use super::*;

pub(super) fn context(
    spec: &ProjectSpec,
    snapshot: &BTreeMap<String, String>,
) -> Result<Vec<serde_json::Value>> {
    let count: usize = spec.supporting_spans.values().map(Vec::len).sum();
    ensure!(count <= 8, "at most eight supporting spans");
    let mut output = vec![];
    let mut bytes = 0;
    for (path, spans) in &spec.supporting_spans {
        ensure!(
            valid_path(path) && !spans.is_empty(),
            "invalid supporting path/spans"
        );
        ensure!(
            !path.split('/').any(|p| matches!(p, "tests" | "fixtures")),
            "test/fixture paths cannot be supporting context"
        );
        let original = spec
            .protected
            .get(path)
            .or_else(|| spec.editable.get(path))
            .ok_or_else(|| anyhow::anyhow!("supporting path not admitted"))?;
        let source = snapshot
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("supporting source missing"))?;
        // Read authority cannot make mutable bytes a stable supporting observation.
        if spec.protected.contains_key(path) {
            ensure!(source == original, "protected supporting source changed");
        } else {
            ensure!(
                spec.edit_windows.contains_key(path),
                "editable supporting source requires an immutable outside window"
            );
            visible_source(spec, path, source)?;
        }
        let mut previous_end = 0;
        for span in spans {
            ensure!(
                span.start < span.end
                    && span.start >= previous_end
                    && original.is_char_boundary(span.start)
                    && original.is_char_boundary(span.end),
                "supporting spans must be sorted, disjoint UTF-8 ranges"
            );
            previous_end = span.end;
            ensure!(
                span.end - span.start <= 2048,
                "supporting span exceeds 2048 bytes"
            );
            let (start, end) = if let Some(window) = spec.edit_windows.get(path) {
                ensure!(
                    span.end <= window.start || span.start >= window.end,
                    "supporting span overlaps edit authority"
                );
                if span.start >= window.end {
                    // Immutable suffix anchored to EOF survives variable-length in-window repairs.
                    (
                        source.len() - (original.len() - span.start),
                        source.len() - (original.len() - span.end),
                    )
                } else {
                    (span.start, span.end)
                }
            } else {
                (span.start, span.end)
            };
            let body = source
                .get(start..end)
                .ok_or_else(|| anyhow::anyhow!("supporting range changed"))?;
            ensure!(
                body == &original[span.start..span.end],
                "supporting bytes changed"
            );
            ensure!(
                !body.contains("#[test]") && !body.contains("#[cfg(test)]"),
                "test declarations cannot be supporting context"
            );
            bytes += body.len();
            ensure!(bytes <= 4096, "supporting context exceeds 4096 bytes");
            output.push(serde_json::json!({"path":path,"editable":false,"source":body,
                "source_digest":Hash::of(&[source.as_bytes()]),"span_digest":Hash::of(&[body.as_bytes()]),
                "initial_start":span.start,"initial_end":span.end,"current_start":start,"current_end":end,
                "authority":"read-only untrusted observation; no patch handle or new permission"}));
        }
    }
    Ok(output)
}
