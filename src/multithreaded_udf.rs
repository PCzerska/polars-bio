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

use log::{info, debug, error};



const BASES: [u8; 5] = [b'A', b'C', b'G', b'T', b'N'];

pub fn create_multithreaded_udf() -> AggregateUDF {
    let accumulator_creator = |_args: AccumulatorArgs<'_>| -> Result<Box<dyn Accumulator>> {
        Ok(Box::new(BaseContentMultiAccumulator::new()))
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
        "base_content_multithreaded",
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
        Arc::new(vec![
            DataType::List(Arc::new(Field::new(
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
            )))
        ]),
    )
}

#[derive(Debug)]
struct BaseContentMultiAccumulator {
    // Vec indexed by position, each is a count map
    content: Vec<HashMap<u8, i64>>,
}

impl BaseContentMultiAccumulator {
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

impl Accumulator for BaseContentMultiAccumulator {
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

    fn merge_batch(&mut self, states: &[ArrayRef]) -> Result<()> {
        if states.is_empty() {
            return Ok(());
        }

        let list_array = states[0]
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| DataFusionError::Execution("Expected ListArray in merge_batch".into()))?;

        let struct_array = list_array.values()
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| DataFusionError::Execution("Expected StructArray inside ListArray".into()))?;

        let pos_array = struct_array.column(0)
            .as_any()
            .downcast_ref::<arrow_array::Int64Array>()
            .ok_or_else(|| DataFusionError::Execution("Expected Int64Array for position".into()))?;

        for row in 0..struct_array.len() {
            let pos = pos_array.value(row) as usize - 1;

            if self.content.len() <= pos {
                self.content.resize_with(pos + 1, HashMap::new);
            }

            let map = self.content.get_mut(pos).unwrap();
            for (i, &base) in BASES.iter().enumerate() {
                let array = struct_array.column(i + 1)
                    .as_any()
                    .downcast_ref::<arrow_array::Int64Array>()
                    .ok_or_else(|| DataFusionError::Execution("Expected Int64Array for base".into()))?;
                let value = array.value(row);
                *map.entry(base).or_insert(0) += value;
            }
        }

        Ok(())
    }

    fn state(&mut self) -> Result<Vec<ScalarValue>> {
        let mut position_builder = Int64Builder::new();
        let mut a_builder = Int64Builder::new();
        let mut c_builder = Int64Builder::new();
        let mut g_builder = Int64Builder::new();
        let mut t_builder = Int64Builder::new();
        let mut n_builder = Int64Builder::new();

        for (i, map) in self.content.iter().enumerate() {
            position_builder.append_value((i + 1) as i64);
            a_builder.append_value(*map.get(&b'A').unwrap_or(&0));
            c_builder.append_value(*map.get(&b'C').unwrap_or(&0));
            g_builder.append_value(*map.get(&b'G').unwrap_or(&0));
            t_builder.append_value(*map.get(&b'T').unwrap_or(&0));
            n_builder.append_value(*map.get(&b'N').unwrap_or(&0));
        }

        let struct_array = StructArray::from(vec![
            (
                Arc::new(Field::new("position", DataType::Int64, false)),
                Arc::new(position_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("A", DataType::Int64, false)),
                Arc::new(a_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("C", DataType::Int64, false)),
                Arc::new(c_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("G", DataType::Int64, false)),
                Arc::new(g_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("T", DataType::Int64, false)),
                Arc::new(t_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("N", DataType::Int64, false)),
                Arc::new(n_builder.finish()) as ArrayRef,
            ),
        ]);

        let list_array = ListArray::new(
            Arc::new(Field::new("item", struct_array.data_type().clone(), false)),
            OffsetBuffer::new(arrow_buffer::ScalarBuffer::from(vec![0i32, struct_array.len() as i32])),
            Arc::new(struct_array),
            None,
        );

        Ok(vec![ScalarValue::List(Arc::new(list_array))])
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

        let struct_array = StructArray::from(vec![
            (
                Arc::new(Field::new("position", DataType::Int64, false)),
                Arc::new(position_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("A", DataType::Int64, false)),
                Arc::new(a_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("C", DataType::Int64, false)),
                Arc::new(c_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("G", DataType::Int64, false)),
                Arc::new(g_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("T", DataType::Int64, false)),
                Arc::new(t_builder.finish()) as ArrayRef,
            ),
            (
                Arc::new(Field::new("N", DataType::Int64, false)),
                Arc::new(n_builder.finish()) as ArrayRef,
            ),
        ]);


        let list_array = ListArray::new(
            Arc::new(Field::new("item", struct_array.data_type().clone(), false)),
            OffsetBuffer::new(arrow_buffer::ScalarBuffer::from(vec![0i32, struct_array.len() as i32])),
            Arc::new(struct_array),
            None,
        );


        Ok(ScalarValue::List(Arc::new(list_array)))
    }



    fn size(&self) -> usize {
        self.content.len() * std::mem::size_of::<HashMap<u8, i64>>() + std::mem::size_of::<Self>()
    }
}