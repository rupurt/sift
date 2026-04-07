# World Model: The Physics of Sift

This document defines the conceptual framework and "physics" that govern the design and evolution of Sift.

## The Core Analogy

Sift is not a file searcher; it is a **High-Energy Information Reactor**. That
still allows path-shaped intent to influence retrieval as structural evidence
without turning the reactor into an editor picker or a second planner.

### 1. The Corpus is Mass
Data in its raw state (files on disk, conversational logs) is inert **Mass**. It has volume and density but no utility until acted upon.

### 2. Retrieval is Magnetism
Retrieval is the force of **Attraction**. 
- **The Query is the Magnet:** It generates a field that pulls relevant Mass into the containment area.
- **Retrieval Strategies are Field Configurations:** BM25, phrase matching, path-fuzzy recall, segment-fuzzy evidence, and vector similarity are different ways to align the magnetic field to pull the most "charged" mass.
- **Agentic Search is Pulsed Magnetism:** A controller can retune the field turn by turn, decomposing a hard query into smaller pulls instead of relying on one pass.
- **The Shortlist is the Containment Field:** We pull raw mass into a tight, manageable volume (the shortlist) where it can be reacted upon.

### 3. Emission is Fusion
The transition from a list of candidates to a final result is a **Fusion Reaction**.
- **Compression:** The Reranker compresses the containment field, forcing the most relevant candidates into close proximity.
- **Reaction:** The LLM acts as the catalyst, reacting with the compressed candidates to release "Energy" (Intent-aligned value).
- **Context Management:** An agentic controller can vent or retain energy between turns, pruning stale evidence to keep the reactor stable.
- **The Emission Port:** The Reactor must have configurable ports to bleed off different types of energy:
    - **Latent Emission:** Raw embedding vectors (The signal before it is formed).
    - **Protocol Emission:** Structured domain records (Turns, Logic Lineage).
    - **Visual Emission:** Rendered views for human consumption (The "Light" of the reaction).

## Strategic Axioms

1.  **Energy over Entropy:** Every step in the search graph must increase the "Signal-to-Noise Gain." If a fusion step (reranking) consumes more compute than the value it adds, it is a failed reaction.
2.  **Containment Efficiency:** A perfect reactor requires minimal "Mass" (shortlist size) to achieve a high-energy "Reaction" (relevant result). We measure the efficiency of our Magnetism by how much Plasma Density (Compression) we can achieve without losing the signal.
3.  **Multi-Modal Transmutation:** The Reactor is agnostic to the final form of energy. It can transmute raw File Mass into Agentic Turn Energy just as easily as it transmutates Query Mass into Latent Vector Signal.
4.  **Turns Must Be Explicit:** Agentic behavior is not hidden magic. Every decomposition, retrieval pulse, prune step, and emission should be inspectable as part of the reactor trace.

## Implications for Development

- **The API is a Reactor Control Panel:** Developers should be configuring fields, turns, and ports, not just calling "search."
- **Evals are Gauges:** We don't just measure "proximity" (MRR); we measure "Reactor Yield" (Signal Gain), turn efficiency, and "Protocol Integrity" (Emission Fidelity).
