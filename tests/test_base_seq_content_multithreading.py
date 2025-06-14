
import time
import matplotlib.pyplot as plt
import polars_bio as pb
from polars_bio import read_fastq

# file_path = "../polars-bio/ERR194147.fastq"
# table_name = "ERR194147"

file_path = "../polars-bio/tbd-project/example.fastq"
table_name = "example"

read_fastq(file_path)

pb.ctx.set_option("datafusion.execution.target_partitions", "2")
start_multi = time.time()
df_multi = pb.sql(f"SELECT base_content_multithreaded(sequence) AS result FROM {table_name}").collect()
multi_time = time.time() - start_multi
print(f"[Multithreaded] Execution time: {multi_time:.4f}s")

pb.ctx.set_option("datafusion.execution.target_partitions", "1")
start_single = time.time()
df_single = pb.sql(f"SELECT base_content(sequence) AS result FROM {table_name}").collect()
single_time = time.time() - start_single
print(f"[Single-threaded] Execution time: {single_time:.4f}s")

labels = ["Multithreaded", "Single-threaded"]
times = [multi_time, single_time]

plt.figure(figsize=(6, 4))
plt.bar(labels, times, color=["#4caf50", "#f44336"])
plt.ylabel("Execution Time (s)")
plt.title("Multithreading Performance (base_content)")
plt.tight_layout()
plt.savefig("multithreading_example_benchmark.png")
print("Saved plot to multithreading_benchmark.png")
plt.show()

# tolerance = 0.10  
# assert multi_time <= single_time * (1 + tolerance), (
#     f"Multithreaded version slower: {multi_time:.4f}s vs {single_time:.4f}s"
# )
