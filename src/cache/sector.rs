use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cache::{corpus_cache_key, load_locked_bincode_default, save_locked_bincode};

pub const SECTOR_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SECTOR_BUCKETS: u16 = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorMap {
    pub schema_version: u32,
    pub corpus_key: String,
    pub partition: SectorPartition,
    pub sectors: Vec<SectorRecord>,
}

impl Default for SectorMap {
    fn default() -> Self {
        Self {
            schema_version: SECTOR_SCHEMA_VERSION,
            corpus_key: String::new(),
            partition: SectorPartition::default(),
            sectors: Vec::new(),
        }
    }
}

impl SectorMap {
    pub fn build_for_root<I>(
        root: &Path,
        members: I,
        strategy: SectorPartitionStrategy,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = SectorMemberInput>,
    {
        let corpus_key = corpus_cache_key(root);
        let mut buckets = BTreeMap::<u16, Vec<SectorProofMaterial>>::new();
        let mut member_count = 0usize;

        for member in members {
            let proof = member.into_proof(root);
            let bucket = strategy.bucket_for_member(&corpus_key, &proof.member_key);
            buckets.entry(bucket).or_default().push(proof);
            member_count += 1;
        }

        let sectors = buckets
            .into_iter()
            .map(|(bucket_ordinal, mut proofs)| {
                proofs.sort_by(|left, right| {
                    left.member_key
                        .cmp(&right.member_key)
                        .then(left.relative_path.cmp(&right.relative_path))
                        .then(left.artifact_blob_key.cmp(&right.artifact_blob_key))
                });

                SectorRecord {
                    sector_id: sector_id_for_bucket(&corpus_key, strategy, bucket_ordinal),
                    bucket_ordinal,
                    membership: SectorMembershipSummary::from_proofs(&proofs),
                    proofs,
                    shards: SectorShardRefs::default(),
                }
            })
            .collect();

        Ok(Self {
            schema_version: SECTOR_SCHEMA_VERSION,
            corpus_key,
            partition: SectorPartition {
                strategy,
                member_count,
            },
            sectors,
        })
    }

    pub fn load(path: &Path) -> Result<Self> {
        load_locked_bincode_default(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        save_locked_bincode(path, self)
    }

    pub fn load_for_root(cache_base: &Path, root: &Path) -> Result<Self> {
        Self::load(&sector_map_cache_path(cache_base, root))
    }

    pub fn save_for_root(&self, cache_base: &Path, root: &Path) -> Result<()> {
        self.save(&sector_map_cache_path(cache_base, root))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorPartition {
    pub strategy: SectorPartitionStrategy,
    pub member_count: usize,
}

impl Default for SectorPartition {
    fn default() -> Self {
        Self {
            strategy: SectorPartitionStrategy::default(),
            member_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectorPartitionStrategy {
    StableHash { bucket_count: u16 },
}

impl Default for SectorPartitionStrategy {
    fn default() -> Self {
        Self::stable_hash(DEFAULT_SECTOR_BUCKETS)
    }
}

impl SectorPartitionStrategy {
    pub const fn stable_hash(bucket_count: u16) -> Self {
        Self::StableHash { bucket_count }
    }

    pub fn bucket_count(self) -> u16 {
        match self {
            Self::StableHash { bucket_count } => bucket_count.max(1),
        }
    }

    fn bucket_for_member(self, corpus_key: &str, member_key: &str) -> u16 {
        let material = format!(
            "sector-bucket-v{}\n{}\n{}",
            SECTOR_SCHEMA_VERSION, corpus_key, member_key
        );
        let hash = blake3::hash(material.as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash.as_bytes()[..8]);
        let value = u64::from_le_bytes(bytes);
        (value % self.bucket_count() as u64) as u16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorRecord {
    pub sector_id: String,
    pub bucket_ordinal: u16,
    pub membership: SectorMembershipSummary,
    pub proofs: Vec<SectorProofMaterial>,
    pub shards: SectorShardRefs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorMembershipSummary {
    pub member_count: usize,
    pub first_member_key: Option<String>,
    pub last_member_key: Option<String>,
    pub proof_fingerprint: String,
}

impl SectorMembershipSummary {
    fn from_proofs(proofs: &[SectorProofMaterial]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for proof in proofs {
            hasher.update(proof.member_key.as_bytes());
            hasher.update(stable_path_key(&proof.relative_path).as_bytes());
            hasher.update(&[proof.proof_kind as u8]);
            hasher.update(proof.artifact_blob_key.as_bytes());
        }

        Self {
            member_count: proofs.len(),
            first_member_key: proofs.first().map(|proof| proof.member_key.clone()),
            last_member_key: proofs.last().map(|proof| proof.member_key.clone()),
            proof_fingerprint: hasher.finalize().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum SectorProofKind {
    FileSystem = 1,
    Synthetic = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorProofMaterial {
    pub member_key: String,
    pub relative_path: PathBuf,
    pub proof_kind: SectorProofKind,
    pub artifact_blob_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorMemberInput {
    pub path: PathBuf,
    pub proof_kind: SectorProofKind,
    pub artifact_blob_key: String,
}

impl SectorMemberInput {
    pub fn filesystem(path: impl Into<PathBuf>, artifact_blob_key: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            proof_kind: SectorProofKind::FileSystem,
            artifact_blob_key: artifact_blob_key.into(),
        }
    }

    pub fn synthetic(path: impl Into<PathBuf>, artifact_blob_key: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            proof_kind: SectorProofKind::Synthetic,
            artifact_blob_key: artifact_blob_key.into(),
        }
    }

    fn into_proof(self, root: &Path) -> SectorProofMaterial {
        let relative_path = relative_member_path(root, &self.path);
        let member_key = stable_path_key(&relative_path);

        SectorProofMaterial {
            member_key,
            relative_path,
            proof_kind: self.proof_kind,
            artifact_blob_key: self.artifact_blob_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SectorShardRefs {
    pub bm25: Option<SectorLexicalShardRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectorLexicalShardRef {
    pub format: SectorLexicalShardFormat,
    pub key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectorLexicalShardFormat {
    Bm25Bincode,
}

pub fn sector_map_cache_path(cache_base: &Path, root: &Path) -> PathBuf {
    cache_base
        .join("artifacts")
        .join("sectors")
        .join(format!("{}.bin", corpus_cache_key(root)))
}

pub fn sector_bm25_shard_cache_path(
    cache_base: &Path,
    root: &Path,
    sector_id: &str,
    shard_key: &str,
) -> PathBuf {
    cache_base
        .join("artifacts")
        .join("indexes")
        .join(corpus_cache_key(root))
        .join("sectors")
        .join(sector_id)
        .join(format!("{}.bin", shard_key))
}

fn sector_id_for_bucket(
    corpus_key: &str,
    strategy: SectorPartitionStrategy,
    bucket_ordinal: u16,
) -> String {
    let material = format!(
        "sector-id-v{}\n{}\n{}\n{}",
        SECTOR_SCHEMA_VERSION,
        corpus_key,
        strategy.bucket_count(),
        bucket_ordinal
    );
    blake3::hash(material.as_bytes()).to_string()
}

fn relative_member_path(root: &Path, path: &Path) -> PathBuf {
    if root.is_file() && root == path {
        return root
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf());
    }

    if let Ok(relative) = path.strip_prefix(root)
        && !relative.as_os_str().is_empty()
    {
        return relative.to_path_buf();
    }

    path.to_path_buf()
}

fn stable_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    use super::{
        SectorMap, SectorMemberInput, SectorPartitionStrategy, sector_bm25_shard_cache_path,
        sector_map_cache_path,
    };
    use crate::cache::corpus_cache_key;

    #[test]
    fn sector_partitioning_is_deterministic_across_input_order() {
        let root = Path::new("/repo");
        let strategy = SectorPartitionStrategy::stable_hash(8);
        let ordered = vec![
            SectorMemberInput::filesystem("/repo/src/main.rs", "hash-main"),
            SectorMemberInput::filesystem("/repo/src/lib.rs", "hash-lib"),
            SectorMemberInput::filesystem("/repo/tests/search.rs", "hash-test"),
            SectorMemberInput::synthetic(".sift/context/environment/branch.txt", "hash-branch"),
        ];
        let reversed = ordered.iter().cloned().rev().collect::<Vec<_>>();

        let left = SectorMap::build_for_root(root, ordered, strategy).expect("build ordered map");
        let right =
            SectorMap::build_for_root(root, reversed, strategy).expect("build reversed map");

        assert_eq!(left, right);
    }

    #[test]
    fn membership_fingerprint_changes_when_proof_material_changes() {
        let root = Path::new("/repo");
        let strategy = SectorPartitionStrategy::stable_hash(1);
        let baseline = SectorMap::build_for_root(
            root,
            vec![
                SectorMemberInput::filesystem("/repo/src/main.rs", "hash-main-v1"),
                SectorMemberInput::filesystem("/repo/src/lib.rs", "hash-lib"),
            ],
            strategy,
        )
        .expect("build baseline map");
        let changed = SectorMap::build_for_root(
            root,
            vec![
                SectorMemberInput::filesystem("/repo/src/main.rs", "hash-main-v2"),
                SectorMemberInput::filesystem("/repo/src/lib.rs", "hash-lib"),
            ],
            strategy,
        )
        .expect("build changed map");

        assert_eq!(baseline.sectors.len(), 1);
        assert_eq!(changed.sectors.len(), 1);
        assert_ne!(
            baseline.sectors[0].membership.proof_fingerprint,
            changed.sectors[0].membership.proof_fingerprint
        );
    }

    #[test]
    fn sector_map_round_trips_through_locked_bincode_storage() {
        let root = Path::new("/repo");
        let strategy = SectorPartitionStrategy::stable_hash(4);
        let map = SectorMap::build_for_root(
            root,
            vec![
                SectorMemberInput::filesystem("/repo/src/main.rs", "hash-main"),
                SectorMemberInput::filesystem("/repo/src/lib.rs", "hash-lib"),
            ],
            strategy,
        )
        .expect("build map");
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("sector-map.bin");

        map.save(&path).expect("save sector map");
        let loaded = SectorMap::load(&path).expect("load sector map");

        assert_eq!(loaded, map);
    }

    #[test]
    fn sector_cache_paths_extend_the_existing_corpus_key_layout() {
        let cache_base = Path::new("/tmp/sift-cache");
        let root = Path::new("/repo");
        let corpus_key = corpus_cache_key(root);
        let sector_map_path = sector_map_cache_path(cache_base, root);
        let shard_path =
            sector_bm25_shard_cache_path(cache_base, root, "sector-01", "bm25-shard-signature");

        assert_eq!(
            sector_map_path,
            PathBuf::from("/tmp/sift-cache")
                .join("artifacts")
                .join("sectors")
                .join(format!("{corpus_key}.bin"))
        );
        assert_eq!(
            shard_path,
            PathBuf::from("/tmp/sift-cache")
                .join("artifacts")
                .join("indexes")
                .join(corpus_key)
                .join("sectors")
                .join("sector-01")
                .join("bm25-shard-signature.bin")
        );
    }
}
