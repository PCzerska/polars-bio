use arrow::array::{StringArray, ArrayRef};
use arrow::datatypes::{Field, DataType, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

mod base_sequence_content;
use base_sequence_content::compute_base_sequence_content;

fn main() {
    let seq_data = vec![
        Some("ATCGATCG".to_string()),
        Some("GGGAAAAT".to_string()),
        Some("NNNNCCCC".to_string()),
    ];

    let seq_array: ArrayRef = Arc::new(StringArray::from(seq_data));

    let schema = Arc::new(Schema::new(vec![
        Field::new("seq", DataType::Utf8, true),
    ]));

    let batch = RecordBatch::try_new(schema, vec![seq_array]).unwrap();

    let df = compute_base_sequence_content(&batch);

    println!("{}", df);
}
