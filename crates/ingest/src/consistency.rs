use spec_db_core::SpecDbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencySnapshot {
    pub source: String,
    pub git_sha: Option<String>,
    pub doc_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyStatus {
    InSync,
    Drift { sha_mismatch: bool, count_mismatch: bool },
    NeverSynced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyReport {
    pub status: ConsistencyStatus,
    pub tantivy: ConsistencySnapshot,
    pub fjall: ConsistencySnapshot,
}

pub fn verify_cross_store_consistency(
    fjall_sha: Option<String>,
    fjall_count: Option<usize>,
    tantivy_sha: Option<String>,
    tantivy_count: Option<usize>,
) -> Result<ConsistencyReport, SpecDbError> {
    let status = if fjall_sha.is_none() && tantivy_sha.is_none() {
        ConsistencyStatus::NeverSynced
    } else {
        let sha_mismatch = fjall_sha != tantivy_sha;
        let count_mismatch = fjall_count != tantivy_count;
        if !sha_mismatch && !count_mismatch {
            ConsistencyStatus::InSync
        } else {
            ConsistencyStatus::Drift { sha_mismatch, count_mismatch }
        }
    };

    Ok(ConsistencyReport {
        status,
        tantivy: ConsistencySnapshot {
            source: "tantivy".to_string(),
            git_sha: tantivy_sha,
            doc_count: tantivy_count,
        },
        fjall: ConsistencySnapshot {
            source: "fjall".to_string(),
            git_sha: fjall_sha,
            doc_count: fjall_count,
        },
    })
}

pub fn verify_consistency(
    fjall_sha: Option<String>,
    fjall_count: Option<usize>,
    tantivy_sha: Option<String>,
    tantivy_count: Option<usize>,
) -> ConsistencyReport {
    let status = if fjall_sha.is_none() && tantivy_sha.is_none() {
        ConsistencyStatus::NeverSynced
    } else {
        let sha_mismatch = fjall_sha != tantivy_sha;
        let count_mismatch = fjall_count != tantivy_count;
        if !sha_mismatch && !count_mismatch {
            ConsistencyStatus::InSync
        } else {
            ConsistencyStatus::Drift { sha_mismatch, count_mismatch }
        }
    };

    ConsistencyReport {
        status,
        tantivy: ConsistencySnapshot {
            source: "tantivy".to_string(),
            git_sha: tantivy_sha,
            doc_count: tantivy_count,
        },
        fjall: ConsistencySnapshot {
            source: "fjall".to_string(),
            git_sha: fjall_sha,
            doc_count: fjall_count,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsistencyStatus, verify_cross_store_consistency};

    #[test]
    fn in_sync_when_matching() {
        let report = verify_cross_store_consistency(
            Some("abc123".to_string()),
            Some(10),
            Some("abc123".to_string()),
            Some(10),
        )
        .unwrap();
        assert_eq!(report.status, ConsistencyStatus::InSync);
    }

    #[test]
    fn drift_on_sha_mismatch() {
        let report = verify_cross_store_consistency(
            Some("abc123".to_string()),
            Some(10),
            Some("def456".to_string()),
            Some(10),
        )
        .unwrap();
        assert_eq!(
            report.status,
            ConsistencyStatus::Drift { sha_mismatch: true, count_mismatch: false }
        );
    }

    #[test]
    fn drift_on_count_mismatch() {
        let report = verify_cross_store_consistency(
            Some("abc123".to_string()),
            Some(9),
            Some("abc123".to_string()),
            Some(10),
        )
        .unwrap();
        assert_eq!(
            report.status,
            ConsistencyStatus::Drift { sha_mismatch: false, count_mismatch: true }
        );
    }

    #[test]
    fn never_synced_when_both_none() {
        let report = verify_cross_store_consistency(None, None, None, None).unwrap();
        assert_eq!(report.status, ConsistencyStatus::NeverSynced);
    }
}
