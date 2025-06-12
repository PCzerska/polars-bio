use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::{ArrayRef, StringArray, StructArray};
use arrow_array::builder::Int64Builder;
use arrow_schema::{DataType, Field, Fields};
use datafusion::common::{Result, DataFusionError};
use datafusion::logical_expr::{Volatility, AggregateUDF, create_udaf};
use datafusion::physical_plan::Accumulator;
use datafusion::logical_expr::function::AccumulatorArgs;
use datafusion::scalar::ScalarValue;
use arrow_array::Array;

use arrow_array::array::ListArray;
use arrow_buffer::OffsetBuffer;


pub fn create_base_content_udaf() -> AggregateUDF {
    let accumulator_creator = |_args: AccumulatorArgs<'_>| -> Result<Box<dyn Accumulator>> {
        Ok(Box::new(BaseContentAccumulator::new()))
    };

    let fields = vec![
        Field::new("position", DataType::Int64, false),
        Field::new("A", DataType::Int64, false),
        Field::new("C", DataType::Int64, false),
        Field::new("G", DataType::Int64, false),
        Field::new("T", DataType::Int64, false),
        Field::new("N", DataType::Int64, false),
    ];

    create_udaf(
        "base_content",
        vec![DataType::Utf8],
        Arc::new(DataType::List(Arc::new(Field::new(
            "item",
            DataType::Struct(Fields::from(vec![
                Field::new("position", DataType::Int64, false),
                Field::new("A", DataType::Int64, false),
                Field::new("C", DataType::Int64, false),
                Field::new("G", DataType::Int64, false),
                Field::new("T", DataType::Int64, false),
                Field::new("N", DataType::Int64, false),
            ])),
            false,
        )))),
        Volatility::Immutable,
        Arc::new(accumulator_creator),
        Arc::new(vec![]),
    )
}

#[derive(Debug)]
struct BaseContentAccumulator {
    // Vec indexed by position, each is a count map
    content: Vec<HashMap<u8, i64>>,
}

impl BaseContentAccumulator {
    fn new() -> Self {
        Self { content: vec![] }
    }

    fn update_counts(&mut self, sequence: &[u8]) {
        for (i, base) in sequence.iter().enumerate() {
            if self.content.len() <= i {
                self.content.push(HashMap::new());
            }

            let counts = self.content.get_mut(i).unwrap();
            let base = match base {
                b'A' | b'a' => b'A',
                b'C' | b'c' => b'C',
                b'G' | b'g' => b'G',
                b'T' | b't' => b'T',
                b'N' | b'n' => b'N',
                _ => b'N',
            };
            *counts.entry(base).or_insert(0) += 1;
        }
    }

    fn merge_counts(&mut self, other: &Vec<HashMap<u8, i64>>) {
        if self.content.len() < other.len() {
            self.content.resize_with(other.len(), HashMap::new);
        }

        for (i, other_map) in other.iter().enumerate() {
            let counts = self.content.get_mut(i).unwrap();
            for (&base, &count) in other_map {
                *counts.entry(base).or_insert(0) += count;
            }
        }
    }
}

impl Accumulator for BaseContentAccumulator {
    fn update_batch(&mut self, values: &[ArrayRef]) -> Result<()> {
        let seq_array = values[0]
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| DataFusionError::Execution("Expected StringArray".into()))?;

        for i in 0..seq_array.len() {
            if seq_array.is_null(i) {
                continue;
            }
            let seq = seq_array.value(i).as_bytes();
            self.update_counts(seq);
        }

        Ok(())
    }

    fn merge_batch(&mut self, _states: &[ArrayRef]) -> Result<()> {
        // No merge state yet (simple implementation), so no-op
        Ok(())
    }

    fn state(&mut self) -> Result<Vec<ScalarValue>> {
        Ok(vec![]) // Not used for now
    }


    fn evaluate(&mut self) -> Result<ScalarValue> {

        let mut position_builder = Int64Builder::new();
        let mut a_builder = Int64Builder::new();
        let mut c_builder = Int64Builder::new();
        let mut g_builder = Int64Builder::new();
        let mut t_builder = Int64Builder::new();
        let mut n_builder = Int64Builder::new();
    
        for (i, count_map) in self.content.iter().enumerate() {
            position_builder.append_value(i as i64 + 1);
            a_builder.append_value(*count_map.get(&b'A').unwrap_or(&0));
            c_builder.append_value(*count_map.get(&b'C').unwrap_or(&0));
            g_builder.append_value(*count_map.get(&b'G').unwrap_or(&0));
            t_builder.append_value(*count_map.get(&b'T').unwrap_or(&0));
            n_builder.append_value(*count_map.get(&b'N').unwrap_or(&0));
        }
    
        let struct_array = StructArray::new(
            Fields::from(vec![
                Field::new("position", DataType::Int64, false),
                Field::new("A", DataType::Int64, false),
                Field::new("C", DataType::Int64, false),
                Field::new("G", DataType::Int64, false),
                Field::new("T", DataType::Int64, false),
                Field::new("N", DataType::Int64, false),
            ]),
            vec![
                Arc::new(position_builder.finish()) as ArrayRef,
                Arc::new(a_builder.finish()),
                Arc::new(c_builder.finish()),
                Arc::new(g_builder.finish()),
                Arc::new(t_builder.finish()),
                Arc::new(n_builder.finish()),
            ],
            None,
        );


        let list_array = ListArray::new(
            Arc::new(Field::new("item", struct_array.data_type().clone(), false)),
            OffsetBuffer::from_lengths([struct_array.len()]),
            Arc::new(struct_array),
            None,
        );

    
        Ok(ScalarValue::List(Arc::new(list_array)))
    }
    


    fn size(&self) -> usize {
        self.content.len() * std::mem::size_of::<HashMap<u8, i64>>() + std::mem::size_of::<Self>()
    }
}