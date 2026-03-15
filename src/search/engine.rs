use anyhow::Result;
use super::domain::{SearchPlan, SearchResponse, LoadedCorpus, Bm25Index, SearchRequest};

/// Abstract persistence layer for search artifacts.
pub trait SearchStorage: Send + Sync {
    fn corpus(&self) -> &LoadedCorpus;
    fn bm25_index(&self) -> &Bm25Index;
}

/// Abstract representation of a search strategy.
/// Initially this wraps the SearchPlan, but will evolve into a Graph IR.
pub trait SearchIR: Send + Sync {
    fn plan(&self) -> &SearchPlan;
}

/// Abstract execution runtime for a search graph.
pub trait SearchExecution: Send + Sync {
    fn execute(
        &self,
        ir: &dyn SearchIR,
        storage: &dyn SearchStorage,
        request: &SearchRequest,
    ) -> Result<SearchResponse>;
}

/// The unified search engine orchestrator.
pub trait SearchEngine: Send + Sync {
    fn search(&self, request: &SearchRequest) -> Result<SearchResponse>;
}

/// A concrete implementation of SearchEngine that ties the traits together.
pub struct GenericEngine<IR, Exec, Storage>
where
    IR: SearchIR,
    Exec: SearchExecution,
    Storage: SearchStorage,
{
    pub ir: IR,
    pub execution: Exec,
    pub storage: Storage,
}

impl<IR, Exec, Storage> SearchEngine for GenericEngine<IR, Exec, Storage>
where
    IR: SearchIR,
    Exec: SearchExecution,
    Storage: SearchStorage,
{
    fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        self.execution.execute(&self.ir, &self.storage, request)
    }
}
