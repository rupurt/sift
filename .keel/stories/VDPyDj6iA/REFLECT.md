# Reflection - Reduce Search Pipeline Memory Allocations

We optimized the critical paths of search execution by introducing pre-allocation for vectors and using lifetime-bound references in `DocAccumulator` during the aggregation phase. These changes eliminate most of the overhead associated with heap allocations in the inner loop of `score_segments_manually` and `aggregate_segment_hits`, ensuring stable performance and memory usage even for large corpora.
