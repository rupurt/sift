# Reflect - Add Query Expansion And Phrase Retrieval (1vzdDn000)

## What was learned?
- Orchestrating multiple query variants required the `Retriever` trait to handle a slice of `QueryVariant` rather than a single string.
- Decoupling retrievers from each other and the fusion logic simplifies the addition of new retrieval strategies.
- `PhraseRetriever` can be efficiently implemented over the transient `Document` model without needing a full positional index for now.

## Any surprises?
- The RRF scores vary based on the number of retrievers that "see" a document, which is expected but requires careful baseline comparison.

## Future improvements?
- Implement a real synonym expansion using a lexicon or LLM.
- Optimize `PhraseRetriever` with an inverted index if the corpus grows significantly.
