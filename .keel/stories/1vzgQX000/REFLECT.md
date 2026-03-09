# Reflection - Add Embedding To Segment Model

Added `embedding: Option<Vec<f32>>` to the `Segment` domain model. Initialized it to `None` in the segment builder. Removed the `Eq` derive as `f32` vectors do not implement total equality.
