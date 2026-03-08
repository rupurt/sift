---
created_at: 2026-03-08T16:16:27
---

# Reflection - Rewrite README For Current CLI Contract

## Knowledge

- [1vzNKa000](../../knowledge/1vzNKa000.md) Avoid literal commas inside Keel verify command fields

## Observations

The README rewrite moved quickly once the current contract was anchored to code and existing evidence instead of the old marketing thesis. `src/main.rs` plus prior Keel evidence logs were enough to rebuild the public story without touching runtime code.

The main surprise was not product behavior but board mechanics. The initial AC4 verify command failed only because the proof string contained `5,183`, and Keel parsed the comma as annotation structure rather than shell content. That is precisely the kind of workflow edge that should be captured as reusable process knowledge because it looks like a content failure at first glance.
