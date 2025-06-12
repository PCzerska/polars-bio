import polars_bio as pb
import pandas as pd
from polars_bio.io import read_fastq
import polars as pl


if __name__ == "__main__":
    file_name = "example"
    
    read_fastq(file_name + ".fastq")

    print(pb.sql(f"SELECT * FROM {file_name}").collect())

    df = pb.sql(f"SELECT base_content(sequence) AS result FROM {file_name}").collect()

    # Explode the `result` list of structs into separate rows
    exploded = df.explode("result")

    # Convert the struct into separate columns
    flat = exploded.select([
        pl.col("result").struct.field("position"),
        pl.col("result").struct.field("A"),
        pl.col("result").struct.field("C"),
        pl.col("result").struct.field("G"),
        pl.col("result").struct.field("T"),
        pl.col("result").struct.field("N"),
    ])

    print(flat)