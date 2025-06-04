use arrow::array::StringArray;
use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use std::collections::HashMap;

pub fn compute_base_sequence_content(rb: &RecordBatch) -> DataFrame {
    let seq_array = rb
        .column_by_name("seq")
        .expect("SEQ column not found")
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("SEQ column is not a StringArray");

    let mut base_counts: HashMap<usize, HashMap<char, usize>> = HashMap::new();
    let bases = ['A', 'T', 'C', 'G', 'N'];

    seq_array
        .iter()
        .filter_map(|opt_seq| opt_seq)
        .for_each(|seq| {
            for (pos, base) in seq.chars().enumerate() {
                let base = base.to_ascii_uppercase();
                base_counts
                    .entry(pos)
                    .or_insert_with(|| HashMap::from_iter(bases.iter().map(|&b| (b, 0))))
                    .entry(base)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        });

    let mut positions: Vec<u32> = Vec::new();
    let mut a_content: Vec<f64> = Vec::new();
    let mut t_content: Vec<f64> = Vec::new();
    let mut c_content: Vec<f64> = Vec::new();
    let mut g_content: Vec<f64> = Vec::new();
    let mut n_content: Vec<f64> = Vec::new();

    for (pos, counts) in base_counts.iter() {
        let total = counts.values().sum::<usize>() as f64;

        positions.push(*pos as u32);
        a_content.push(*counts.get(&'A').unwrap_or(&0) as f64 / total * 100.0);
        t_content.push(*counts.get(&'T').unwrap_or(&0) as f64 / total * 100.0);
        c_content.push(*counts.get(&'C').unwrap_or(&0) as f64 / total * 100.0);
        g_content.push(*counts.get(&'G').unwrap_or(&0) as f64 / total * 100.0);
        n_content.push(*counts.get(&'N').unwrap_or(&0) as f64 / total * 100.0);
    }

    DataFrame::new(vec![
    Series::new("position".into(), positions).into(),
    Series::new("A(%)".into(), a_content).into(),
    Series::new("T(%)".into(), t_content).into(),
    Series::new("C(%)".into(), c_content).into(),
    Series::new("G(%)".into(), g_content).into(),
    Series::new("N(%)".into(), n_content).into(),

    ])
    .expect("Failed to create DataFrame")
}
