use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::cache::BreadcrumbJournal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FrontierLedger {
    pub total_sector_count: usize,
    pub mounted_sector_count: usize,
    pub reused_sector_count: usize,
    pub dirty_sector_count: usize,
    pub completed_dirty_sector_count: usize,
    pub rebuilding_sector_count: usize,
    pub resumed_sector_count: usize,
    pub active_rebuild: Option<FrontierActiveRebuild>,
    mounted_sector_ids: HashSet<String>,
    reused_sector_ids: HashSet<String>,
    completed_dirty_sector_ids: HashSet<String>,
    resumed_sector_ids: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontierActiveRebuild {
    pub sector_id: String,
    pub next_member_offset: usize,
    pub sector_member_count: usize,
    pub resumed: bool,
}

impl FrontierLedger {
    pub fn new(total_sector_count: usize, dirty_sector_count: usize) -> Self {
        Self {
            total_sector_count,
            dirty_sector_count,
            ..Self::default()
        }
    }

    pub fn record_clean_mount(&mut self, sector_id: impl Into<String>) {
        let sector_id = sector_id.into();
        if self.mounted_sector_ids.insert(sector_id.clone()) {
            self.mounted_sector_count += 1;
            self.dirty_sector_count = self.dirty_sector_count.saturating_sub(1);
        }
        if self.reused_sector_ids.insert(sector_id) {
            self.reused_sector_count += 1;
        }
    }

    pub fn apply_breadcrumb_resume(&mut self, journal: &BreadcrumbJournal) {
        for sector_id in &journal.completed_sectors {
            self.complete_dirty_rebuild(sector_id.clone(), true);
        }

        if let Some(active) = journal.active_sector.as_ref() {
            let resumed = active.next_member_offset > 0;
            self.start_dirty_rebuild(
                active.sector_id.clone(),
                active.sector_member_count,
                resumed,
                active.next_member_offset,
            );
        }
    }

    pub fn start_dirty_rebuild(
        &mut self,
        sector_id: impl Into<String>,
        sector_member_count: usize,
        resumed: bool,
        next_member_offset: usize,
    ) {
        let sector_id = sector_id.into();
        if resumed && self.resumed_sector_ids.insert(sector_id.clone()) {
            self.resumed_sector_count += 1;
        }
        self.rebuilding_sector_count = 1;
        self.active_rebuild = Some(FrontierActiveRebuild {
            sector_id,
            next_member_offset,
            sector_member_count,
            resumed,
        });
    }

    pub fn advance_dirty_rebuild(&mut self, sector_id: &str, next_member_offset: usize) {
        if let Some(active) = self.active_rebuild.as_mut()
            && active.sector_id == sector_id
        {
            active.next_member_offset = next_member_offset;
        }
    }

    pub fn complete_dirty_rebuild(&mut self, sector_id: impl Into<String>, reused: bool) {
        let sector_id = sector_id.into();
        if self.completed_dirty_sector_ids.insert(sector_id.clone()) {
            self.completed_dirty_sector_count += 1;
            self.dirty_sector_count = self.dirty_sector_count.saturating_sub(1);
        }
        if self.mounted_sector_ids.insert(sector_id.clone()) {
            self.mounted_sector_count += 1;
        }
        if reused && self.reused_sector_ids.insert(sector_id.clone()) {
            self.reused_sector_count += 1;
        }
        if self
            .active_rebuild
            .as_ref()
            .is_some_and(|active| active.sector_id == sector_id)
        {
            self.active_rebuild = None;
            self.rebuilding_sector_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrontierActiveRebuild, FrontierLedger};
    use crate::cache::{ActiveBreadcrumbSector, BreadcrumbJournal};

    #[test]
    fn frontier_ledger_tracks_clean_mounts_and_dirty_rebuild_progress() {
        let mut frontier = FrontierLedger::new(4, 4);

        frontier.record_clean_mount("sector-clean");
        frontier.start_dirty_rebuild("sector-dirty-a", 3, false, 0);
        frontier.advance_dirty_rebuild("sector-dirty-a", 2);
        frontier.complete_dirty_rebuild("sector-dirty-a", false);

        assert_eq!(frontier.total_sector_count, 4);
        assert_eq!(frontier.mounted_sector_count, 2);
        assert_eq!(frontier.reused_sector_count, 1);
        assert_eq!(frontier.dirty_sector_count, 2);
        assert_eq!(frontier.completed_dirty_sector_count, 1);
        assert_eq!(frontier.rebuilding_sector_count, 0);
        assert!(frontier.active_rebuild.is_none());
    }

    #[test]
    fn frontier_ledger_derives_resume_state_from_breadcrumbs() {
        let mut frontier = FrontierLedger::new(5, 3);
        let journal = BreadcrumbJournal {
            schema_version: 1,
            corpus_key: "corpus".to_string(),
            run_id: "run-1".to_string(),
            updated_at_unix_secs: 1,
            dirty_sectors: vec![
                "sector-a".to_string(),
                "sector-b".to_string(),
                "sector-c".to_string(),
            ],
            completed_sectors: vec!["sector-a".to_string()],
            active_sector: Some(ActiveBreadcrumbSector {
                sector_id: "sector-b".to_string(),
                next_member_offset: 1,
                next_member_relative_path: Some("doc.txt".into()),
                sector_member_count: 3,
            }),
        };

        frontier.apply_breadcrumb_resume(&journal);

        assert_eq!(frontier.mounted_sector_count, 1);
        assert_eq!(frontier.reused_sector_count, 1);
        assert_eq!(frontier.dirty_sector_count, 2);
        assert_eq!(frontier.completed_dirty_sector_count, 1);
        assert_eq!(frontier.rebuilding_sector_count, 1);
        assert_eq!(frontier.resumed_sector_count, 1);
        assert_eq!(
            frontier.active_rebuild,
            Some(FrontierActiveRebuild {
                sector_id: "sector-b".to_string(),
                next_member_offset: 1,
                sector_member_count: 3,
                resumed: true,
            })
        );
    }
}
