import polars as pl
from polars_bio.polars_bio import py_base_sequence_content  

def base_sequence_content(
    input: pl.DataFrame,
    streaming: bool = True,
    target_partitions: int = None
) -> pl.DataFrame:

    return py_base_sequence_content(input, streaming, target_partitions)
