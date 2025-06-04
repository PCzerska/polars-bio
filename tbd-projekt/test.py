# import polars_bio as pb
# import polars as pl


# pl.Config(fmt_str_lengths=1000, tbl_width_chars=1000)
# pl.Config.set_tbl_cols(100)
# vcf_1 = "gs://gcp-public-data--gnomad/release/4.1/genome_sv/gnomad.v4.1.sv.sites.vcf.gz"
# result = pb.describe_vcf(vcf_1).filter(pl.col("description").str.contains(r"Latino.* allele frequency"))
# print(result)



from polars_bio import read_fastq, add_two_numbers_py, quality_control_py
import polars as pl
from collections import Counter

# def nucleotide_percentages(sequence_series: pl.Series) -> pl.DataFrame:
#     # Połącz wszystkie sekwencje w jeden string
#     combined = "".join(sequence_series.to_list())
    
#     total = len(combined)
#     counts = Counter(combined)
    
#     bases = ['A', 'T', 'C', 'G']
#     percentages = [(counts.get(base, 0) / total) * 100 for base in bases]
    
#     return pl.DataFrame({
#         "base": bases,
#         "percentage": percentages
#     })


# result = add_two_numbers_py(10, 5)
# print(result)

df = read_fastq("example.fastq").collect()
# print(f"TYPE: {type(df)}")
# print(f"sequence type: {type(df["sequence"])}")
#print(df["sequence"])
#result = nucleotide_percentages(df["sequence"])
vec = ["NCAATACAAAAGCAATATGGGAGAAGCTACCTACCATGCTTAAAAACGCCAATGAGCAGNGATTTGTCANCNNNNNNNNCNNNNNNNNTNNTANNANNCTC", "NGTCAAAGATAAGATCAAAAGGCACTGGCTTACCTGATTAAGAAATTGTGTAGTCCAACATCAAAATACNTNTNNNNNAGAGNCANGNCAAGCNNANNAAT"]
#result = quality_control_py(vec)
print(result)




