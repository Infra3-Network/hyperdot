use anyhow::anyhow;
use anyhow::Context;
use anyhow::Ok;
use rust_decimal::prelude::Decimal;
use tokio_postgres::types::FromSql;
use tokio_postgres::types::Type;
use tokio_postgres::Column;
use tokio_postgres::Row;

use crate::types::PostgresColumnData;

fn convert_primitive_type<'a, T: FromSql<'a>>(
    row: &'a Row,
    column: &Column,
    column_i: usize,
    cfn: impl Fn(T) -> Result<serde_json::Value, anyhow::Error>,
) -> Result<serde_json::Value, anyhow::Error> {
    let raw_val = row
        .try_get::<_, Option<T>>(column_i)
        .with_context(|| format!("column_name:{}", column.name()))?;
    raw_val.map_or(Ok(serde_json::Value::Null), cfn)
}

fn convert_array_type<'a, T: FromSql<'a>>(
    row: &'a Row,
    column: &Column,
    column_i: usize,
    cfn: impl Fn(T) -> Result<serde_json::Value, anyhow::Error>,
) -> Result<serde_json::Value, anyhow::Error> {
    let raw_val_array = row
        .try_get::<_, Option<Vec<T>>>(column_i)
        .with_context(|| format!("column_name:{}", column.name()))?;
    Ok(match raw_val_array {
        Some(val_array) => {
            let mut result = vec![];
            for val in val_array {
                result.push(cfn(val)?);
            }
            serde_json::Value::Array(result)
        }
        None => serde_json::Value::Null,
    })
}

// For TS_VECTOR convert
struct StringCollector(String);
impl FromSql<'_> for StringCollector {
    fn from_sql(
        _: &Type,
        raw: &[u8],
    ) -> Result<StringCollector, Box<dyn std::error::Error + Sync + Send>> {
        let result = std::str::from_utf8(raw)?;
        Ok(StringCollector(result.to_owned()))
    }
    fn accepts(_ty: &Type) -> bool {
        true
    }
}

/// Note. return None should skipped.
pub fn to_json_value(
    row: &Row,
    column: &Column,
    column_idx: usize,
) -> Result<Option<PostgresColumnData>, anyhow::Error> {
    let f64_to_json_number = |raw_val: f64| -> Result<serde_json::Value, anyhow::Error> {
        let temp =
            serde_json::Number::from_f64(raw_val.into()).ok_or(anyhow!("invalid json-float"))?;
        Ok(serde_json::Value::Number(temp))
    };

    Ok(match *column.type_() {
        // for rust-postgres <> postgres type-mappings: https://docs.rs/postgres/latest/postgres/types/trait.FromSql.html#types
        // for postgres types: https://www.postgresql.org/docs/7.4/datatype.html#DATATYPE-TABLE
        // single types
        Type::BOOL => convert_primitive_type(row, column, column_idx, |a: bool| {
            Ok(Some(PostgresColumnData {
                column_type: PostgresColumnDataType::Boolean,
                column_value: serde_json::Value::Number(a),
            }))
        })?,

        Type::INT2 => convert_primitive_type(row, column, column_idx, |a: i16| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,

        Type::INT4 => convert_primitive_type(row, column, column_idx, |a: i32| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,

        Type::INT8 => convert_primitive_type(row, column, column_idx, |a: i64| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,

        Type::NUMERIC => {
            let decimal = row
                .try_get::<_, Option<Decimal>>(column_idx)
                .with_context(|| format!("column_name: {}", column.name()))?;

            decimal.map_or(serde_json::Value::Null, |decimal| {
                serde_json::Value::String(decimal.to_string())
            })
        }

        Type::BYTEA => {}

        Type::TEXT | Type::VARCHAR => {
            convert_primitive_type(row, column, column_idx, |a: String| {
                Ok(serde_json::Value::String(a))
            })?
        }
        // Type::JSON | Type::JSONB => get_basic(row, column, column_i, |a: serde_json::Value| Ok(a))?,
        Type::FLOAT4 => convert_primitive_type(row, column, column_idx, |a: f32| {
            Ok(f64_to_json_number(a.into())?)
        })?,
        Type::FLOAT8 => {
            convert_primitive_type(row, column, column_idx, |a: f64| Ok(f64_to_json_number(a)?))?
        }
        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR => {
            convert_primitive_type(row, column, column_idx, |a: StringCollector| {
                Ok(serde_json::Value::String(a.0))
            })?
        }

        // array types
        Type::BOOL_ARRAY => convert_array_type(row, column, column_idx, |a: bool| {
            Ok(serde_json::Value::Bool(a))
        })?,
        Type::INT2_ARRAY => convert_array_type(row, column, column_idx, |a: i16| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,
        Type::INT4_ARRAY => convert_array_type(row, column, column_idx, |a: i32| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,
        Type::INT8_ARRAY => convert_array_type(row, column, column_idx, |a: i64| {
            Ok(serde_json::Value::Number(serde_json::Number::from(a)))
        })?,
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => {
            convert_array_type(row, column, column_idx, |a: String| {
                Ok(serde_json::Value::String(a))
            })?
        }
        Type::JSON_ARRAY | Type::JSONB_ARRAY | Type::JSONB | Type::JSON => {
            unimplemented!("JSON TYPE FAMLIY")
            //    get_array(row, column, column_i, |a: serde_json::Value| Ok(a))?
        }
        Type::FLOAT4_ARRAY => convert_array_type(row, column, column_idx, |a: f32| {
            Ok(f64_to_json_number(a.into())?)
        })?,
        Type::FLOAT8_ARRAY => {
            convert_array_type(row, column, column_idx, |a: f64| Ok(f64_to_json_number(a)?))?
        }
        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR_ARRAY => {
            convert_array_type(row, column, column_idx, |a: StringCollector| {
                Ok(serde_json::Value::String(a.0))
            })?
        }

        _ => anyhow::bail!(
            "Cannot convert pg-cell \"{}\" of type \"{}\" to a serde_json::Value.",
            column.name(),
            column.type_().name()
        ),
    })
}
