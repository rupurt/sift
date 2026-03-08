---
source_type: Story
source: stories/1vzJfp000/REFLECT.md
scope: 1vzJVa000/1vzJda000
source_story_id: 1vzJfp000
created_at: 2026-03-08T14:23:15
---

### 1vzLal000: Separate benchmark IDs from recursive search IDs

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When the same BM25 core is reused for benchmark corpora and raw recursive filesystem search |
| **Insight** | Recursive search needs stable full-path identities to avoid basename collisions, while benchmark corpora still need stem-based IDs to match qrels manifests |
| **Suggested Action** | Keep a shared in-memory document/index layer, but let each loader define its own canonical document ID policy |
| **Applies To** | `src/search.rs`, future hybrid reranking, benchmark corpus loaders |
| **Linked Knowledge IDs** | 1vzLO0001 |
| **Observed At** | 2026-03-08T21:23:30+00:00 |
| **Score** | 0.88 |
| **Confidence** | 0.95 |
| **Applied** | yes |
