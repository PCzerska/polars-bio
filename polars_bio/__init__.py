from polars_bio.polars_bio import InputFormat, ReadOptions, VcfReadOptions
from .base_sequence_content import base_sequence_content

from .context import ctx, set_option
from .io import (
    describe_vcf,
    from_polars,
    read_bam,
    read_fasta,
    read_fastq,
    read_table,
    read_vcf,
    register_vcf,
    register_view,
    sql,
)
from .polars_ext import PolarsRangesOperations as LazyFrame
from .range_op import FilterOp, count_overlaps, coverage, merge, nearest, overlap
from .range_viz import visualize_intervals
# ADDED tbd-projekt test add_two_numbers_py
from .quality_control import add_two_numbers_py



POLARS_BIO_MAX_THREADS = "datafusion.execution.target_partitions"


__version__ = "0.6.3"
__all__ = [
    # START tbd_projekt
    "add_two_numbers_py",
    "base_sequence_content",
    #"quality_control_py",
    # KONIEC tbd_projekt
    "overlap",
    "nearest",
    "merge",
    "count_overlaps",
    "coverage",
    "ctx",
    "FilterOp",
    "visualize_intervals",
    "read_bam",
    "read_vcf",
    "read_fasta",
    "read_fastq",
    "read_table",
    "register_vcf",
    "describe_vcf",
    "register_view",
    "from_polars",
    "sql",
    "InputFormat",
    "LazyFrame",
    "ReadOptions",
    "VcfReadOptions",
    "set_option",
]
