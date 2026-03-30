# Search Progress Callback Interface - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | search_autonomous accepts an optional progress callback parameter enabling downstream consumers to receive phased updates during the blocking call | board: VFO1icY5Z |
| MG-02 | Progress events cover all five phases: Indexing, Embedding, Planning, Retrieving, Ranking with phase-specific counters | board: VFO1icY5Z |
| MG-03 | Each progress event carries an optional estimated_remaining Duration for time-to-completion display | board: VFO1icY5Z |
| MG-04 | Upstream requirements from paddles are documented as a formal bearing for traceability | board: VFO1icY5Z |

## Constraints

- Callback must be zero-cost when not provided (no overhead on existing callers)
- Maintain backward compatibility — existing search_autonomous callers must compile without changes
- Progress types must be public API surface exported from lib.rs
- No async runtime dependency — callback is synchronous Fn, not Future

## Halting Rules

- DO NOT halt while any story in voyage VFO1uSaNE remains incomplete
- DO NOT halt until the paddles requirements bearing is laid and attached
- HALT when all five SearchProgress variants emit correctly through the callback and all stories are done
- YIELD to human if downstream paddles integration reveals requirements not captured in the bearing
