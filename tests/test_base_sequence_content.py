import json
import polars as pl
import polars_bio as pb
from polars_bio.io import read_fastq
import pytest
import time

def test_base_content_matches_expected():
    read_fastq("example.fastq")
    df = pb.sql("SELECT base_content(sequence) AS result FROM example").collect()

    exploded = df.explode("result").select([
        pl.col("result").struct.field("position").alias("pos"),
        pl.col("result").struct.field("A").alias("A"),
        pl.col("result").struct.field("C").alias("C"),
        pl.col("result").struct.field("G").alias("G"),
        pl.col("result").struct.field("T").alias("T"),
        pl.col("result").struct.field("N").alias("N"),
    ])

    melted = exploded.melt(id_vars=["pos"], variable_name="base", value_name="count").sort(["pos", "base"])
    melted = melted.select(["base", "count", "pos"]).sort(["pos", "base"])

    with open("../tests/data/fastqcrs_output/base_seq_content_expected.json", "r") as f:
        expected = pl.DataFrame(json.load(f)["values"]).select(["base", "count", "pos"]).sort(["pos", "base"])
        expected = expected.with_columns([
            (pl.col("pos") + 1).alias("pos") 
        ])


    print("FIRST ROW:")
    for a, b in zip(melted.rows(), expected.rows()):
        if a != b:
            print(f"Got: {a}, Expected: {b}")
            break

    assert melted.rows() == expected.rows(), "Mismatch in base content output"

def test_empty_fastq(tmp_path):
    empty_file = tmp_path / "empty.fastq"
    empty_file.write_text("")
    read_fastq(str(empty_file))
    df = pb.sql("SELECT base_content(sequence) AS result FROM empty").collect()

    assert df.height == 1, "Expected a single row"

    result_series = df.get_column("result")
    result_list = result_series[0]

    assert isinstance(result_list, pl.Series), "Expected result to be a Series"
    assert result_list.is_empty(), "Expected empty list in result"

def test_all_N_fastq(tmp_path):
    fastq_path = tmp_path / "n_only.fastq"
    fastq_path.write_text(
        "@seq1\nNNNN\n+\n!!!!\n@seq2\nNNNN\n+\n!!!!\n"
    )
    read_fastq(str(fastq_path))

    df = pb.sql("SELECT base_content(sequence) AS result FROM n_only").collect()
    exploded = df.explode("result")

    expected = pl.DataFrame({
        "position": [1, 2, 3, 4],
        "A": [0, 0, 0, 0],
        "C": [0, 0, 0, 0],
        "G": [0, 0, 0, 0],
        "T": [0, 0, 0, 0],
        "N": [2, 2, 2, 2]
    })

    actual = exploded.select([
        pl.col("result").struct.field("position"),
        pl.col("result").struct.field("A"),
        pl.col("result").struct.field("C"),
        pl.col("result").struct.field("G"),
        pl.col("result").struct.field("T"),
        pl.col("result").struct.field("N"),
    ])

    for col in expected.columns:
        assert actual.select(col).to_series().to_list() == expected.select(col).to_series().to_list(), f"Mismatch in column {col}"



def test_invalid_characters(tmp_path):
    fastq_path = tmp_path / "invalid.fastq"
    fastq_path.write_text(
        "@seq1\nACGTZ\n+\n!!!!!\n"
    )
    read_fastq(str(fastq_path))

    df = pb.sql("SELECT base_content(sequence) AS result FROM invalid").collect()
    exploded = df.explode("result")

    actual = exploded.select([
        pl.col("result").struct.field("position"),
        pl.col("result").struct.field("A"),
        pl.col("result").struct.field("C"),
        pl.col("result").struct.field("G"),
        pl.col("result").struct.field("T"),
        pl.col("result").struct.field("N"),
    ])

    expected_counts = {
        1: {'A': 1, 'C': 0, 'G': 0, 'T': 0, 'N': 0},
        2: {'A': 0, 'C': 1, 'G': 0, 'T': 0, 'N': 0},
        3: {'A': 0, 'C': 0, 'G': 1, 'T': 0, 'N': 0},
        4: {'A': 0, 'C': 0, 'G': 0, 'T': 1, 'N': 0},
        5: {'A': 0, 'C': 0, 'G': 0, 'T': 0, 'N': 1},  
    }

    for row in actual.iter_rows(named=True):
        pos = row["position"]
        assert row["A"] == expected_counts[pos]["A"]
        assert row["C"] == expected_counts[pos]["C"]
        assert row["G"] == expected_counts[pos]["G"]
        assert row["T"] == expected_counts[pos]["T"]
        assert row["N"] == expected_counts[pos]["N"]

# takes too much time 
# def test_execution_time_multithreading():
#     file_name = "ERR194147"
#     read_fastq("../polars-bio/ERR194147.fastq")

#     pb.ctx.set_option("datafusion.execution.target_partitions", "4")
#     start = time.time()
#     df_multi = pb.sql(f"SELECT base_content_multithreaded(sequence) AS result FROM {file_name}").collect()
#     multi_time = time.time() - start
#     print(f"[Multithreaded] Execution time: {multi_time:.4f}s")

#     pb.ctx.set_option("datafusion.execution.target_partitions", "1")
#     start = time.time()
#     df_single = pb.sql(f"SELECT base_content(sequence) AS result FROM {file_name}").collect()
#     single_time = time.time() - start
#     print(f"[Single-threaded] Execution time: {single_time:.4f}s")

#     tolerance = 0.10  # 10% margines
#     assert multi_time <= single_time * (1 + tolerance), (
#         f"Multithreaded version slower: {multi_time:.4f}s vs {single_time:.4f}s"
#     )
