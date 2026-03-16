# Research: Graph IR and Execution Engine

## Abstract
This document explores the refactoring of `sift` from a structured search pipeline into a modular **Graph IR and Execution Engine**. By elevating **IR**, **Execution**, and **Storage** to formal traits, we transform `sift` from a local CLI tool into a generic retrieval framework suitable for diverse embedding environments.

## The Three Pillars

### 1. SearchEngine (The Orchestrator)
The `SearchEngine` trait serves as the top-level boundary that binds specific IR, Execution, and Storage implementations into a functional "Engine".

```rust
pub trait SearchEngine {
    type IR: SearchIR;
    type Execution: SearchExecution<Self::IR>;
    type Storage: SearchStorage;

    fn ir(&self) -> &Self::IR;
    fn executor(&self) -> &Self::Execution;
    fn storage(&self) -> &Self::Storage;

    /// The unified entry point for search
    fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResponse> {
        let graph = self.ir().compile(query, options)?;
        self.executor().execute(graph, self.storage())
    }
}
```

### 2. SearchIR (The Graph Compiler)
The **Intermediate Representation (IR)** is no longer a static `SearchPlan` struct. It is a trait that "compiles" a user query into an executable **Graph of Operations**.

- **Node:** Represents a discrete operation (e.g., `LexicalRetrieval`, `VectorRetrieval`, `Expansion`).
- **Edge:** Represents the data flow and dependencies (e.g., `CandidateList`).

```rust
pub trait SearchIR {
    type Node;
    type Edge;

    /// Compiles a raw query into an executable Directed Acyclic Graph (DAG)
    fn compile(&self, query: &str, options: SearchOptions) -> Result<SearchGraph<Self::Node, Self::Edge>>;
    
    /// Optimization pass before execution (e.g., pruning, parallel path identification)
    fn optimize(&self, graph: &mut SearchGraph<Self::Node, Self::Edge>);
}
```

### 3. SearchExecution (The Runtime)
The `SearchExecution` trait is responsible for traversing the IR graph. It defines *how* the search runs—whether it's a simple sequential walk, a high-concurrency parallel execution, or an asynchronous task graph.

```rust
pub trait SearchExecution<I: SearchIR> {
    /// Orchestrates the traversal and data flow of the compiled IR graph
    fn execute(&self, graph: SearchGraph<I::Node, I::Edge>, storage: &dyn SearchStorage) -> Result<SearchResponse>;
}
```

### 4. SearchStorage (The Persistence Layer)
Abstracting the **Storage** mechanism allows `sift` to be decoupled from the local filesystem. This is the most "meta" part of the engine, enabling retrieval from S3, databases, or remote APIs.

```rust
pub trait SearchStorage: Send + Sync {
    fn get_document(&self, id: &str) -> Result<Document>;
    fn list_ids(&self, scope: &Path) -> Result<Box<dyn Iterator<Item = String>>>;
    fn get_embedding(&self, doc_id: &str, segment_index: usize) -> Result<Option<Vec<f32>>>;
    fn get_term_frequencies(&self, doc_id: &str) -> Result<HashMap<String, usize>>;
}
```

## Strategic Implications

1. **Universal Retrieval:** Consumers can implement `SearchStorage` for cloud backends (S3, Snowflake) while reusing `sift`'s expansion and fusion logic.
2. **Dynamic IR:** Users can create custom IR compilers that generate complex branching search strategies based on query classification.
3. **Execution Profiling:** By making `Execution` a trait, we can swap in a `TracingExecutor` that records sub-millisecond spans for every node in the graph without polluting the domain logic.

## Current State vs. Vision
Currently, `sift` uses a semi-rigid `SearchPlan` and a hardcoded `execute` loop in `SearchService`. This research proposes transitioning these internal patterns into the public trait-based "Engine" framework to support embedding `sift` as a library across diverse technical stacks.
