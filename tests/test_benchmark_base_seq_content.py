import time
import subprocess
import matplotlib.pyplot as plt
import polars_bio as pb
from polars_bio import read_fastq

datasets = [
    ("../polars-bio/tbd-project/example.fastq", "50KB"),
    ("../polars-bio/ERR194147.fastq", "2GB"),
]
results = []

for file_path, size_label in datasets:
    table_name = file_path.split("/")[-1].split(".")[0]

    read_fastq(file_path)  
    start_my = time.time()
    pb.sql(f"SELECT base_content(sequence) AS result FROM {table_name}").collect()
    time_polars = time.time() - start_my
    print(f"[polars_bio] {file_path} → {time_polars:.2f} sec")

    start_fastqc = time.time()
    subprocess.run(
        ["fqc", "-q", file_path, "-k", "5"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=True
    )
    time_fastqc = time.time() - start_fastqc
    print(f"[fastqc-rs]  {file_path} → {time_fastqc:.2f} sec")

    results.append((size_label, time_polars, time_fastqc))

labels, polars_times, fastqc_times = zip(*results)
x = range(len(labels))
plt.figure(figsize=(8, 5))
plt.bar(x, polars_times, width=0.4, label="polars_bio", align='center')
plt.bar([i + 0.4 for i in x], fastqc_times, width=0.4, label="fastqc-rs", align='center')
plt.xticks([i + 0.2 for i in x], labels)
plt.ylabel("Execution Time (s)")
plt.title("Base Content Performance Comparison")
plt.legend()
plt.tight_layout()
plt.savefig("benchmark_results.png")
print("Saved plot to benchmark_results.png")
