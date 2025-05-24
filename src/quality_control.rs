// // DOCELOWA FUNKCJA tbd_projekt
// use std::sync::Arc;
// use datafusion::arrow::array::{ArrayRef, Float64Array, StringArray};
// use datafusion::arrow::datatypes::DataType;
// use datafusion::common::DataFusionError;
// use datafusion::logical_expr::{create_udf, ScalarUDF, Volatility};
// use datafusion::physical_expr::functions::make_scalar_function;
// use arrow_array::Array;


// fn compute_gc_content(sequence: &str) -> f64 {
//     let gc = sequence.chars().filter(|c| *c == 'G' || *c == 'C' || *c == 'g' || *c == 'c').count();
//     let len = sequence.chars().filter(|c| *c != 'N' && *c != 'n').count();

//     if len == 0 { 0.0 } else { gc as f64 / len as f64 }
// }


// fn gc_content_fn(args: &[ArrayRef]) -> Result<ArrayRef, DataFusionError> {
//     let input = args[0]
//         .as_any()
//         .downcast_ref::<StringArray>()
//         .ok_or_else(|| DataFusionError::Internal("Expected StringArray".to_string()))?;

//     let mut values = Vec::with_capacity(input.len());
//     for i in 0..input.len() {
//         if input.is_null(i) {
//             values.push(None);
//         } else {
//             let v = compute_gc_content(input.value(i));
//             values.push(Some(v));
//         }
//     }

//     let array = Float64Array::from(values);
//     Ok(Arc::new(array))
// }


// pub fn gc_content_udf() -> ScalarUDF {
//     let func: ScalarFunctionImplementation = Arc::new(gc_content_fn);
//     create_udf("gc_content", vec![DataType::Utf8], DataType::Float64, Volatility::Immutable, func)
// }
