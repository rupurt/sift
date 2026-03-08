---
created_at: 2026-03-08T16:17:21
---

# Knowledge - 1vzNFu000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Rewrite README For Current CLI Contract (1vzNHY000)

### 1vzNKa000: Avoid literal commas inside Keel verify command fields

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Authoring `` annotations for stories that need regex or text probes in shell commands |
| **Insight** | Keel's verify annotation parser treats commas as structural separators, so a literal comma inside the command field can corrupt parsing and produce misleading verification failures even when the underlying shell command succeeds. |
| **Suggested Action** | Keep verify command strings comma-free or express the same proof with alternate probes before relying on `keel verify run`. |
| **Applies To** | `.keel/stories/*/README.md`, verification annotations, documentation stories |
| **Applied** | yes |



---

## Synthesis

### z1UQxkCnN: Avoid literal commas inside Keel verify command fields

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Authoring `` annotations for stories that need regex or text probes in shell commands |
| **Insight** | Keel's verify annotation parser treats commas as structural separators, so a literal comma inside the command field can corrupt parsing and produce misleading verification failures even when the underlying shell command succeeds. |
| **Suggested Action** | Keep verify command strings comma-free or express the same proof with alternate probes before relying on `keel verify run`. |
| **Applies To** | `.keel/stories/*/README.md`, verification annotations, documentation stories |
| **Linked Knowledge IDs** | 1vzNKa000 |
| **Score** | 0.76 |
| **Confidence** | 0.93 |
| **Applied** | yes |

