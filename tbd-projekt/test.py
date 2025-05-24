# import polars_bio as pb
# import polars as pl


# pl.Config(fmt_str_lengths=1000, tbl_width_chars=1000)
# pl.Config.set_tbl_cols(100)
# vcf_1 = "gs://gcp-public-data--gnomad/release/4.1/genome_sv/gnomad.v4.1.sv.sites.vcf.gz"
# result = pb.describe_vcf(vcf_1).filter(pl.col("description").str.contains(r"Latino.* allele frequency"))
# print(result)



from polars_bio import read_fastq, add_two_numbers_py

# result = add_two_numbers_py(10, 5)
# print(result)

df = read_fastq("example.fastq").collect()
print(f"TYPE: {type(df)}")
print(f"sequence type: {type(df["sequence"])}")
print(df["sequence"])



