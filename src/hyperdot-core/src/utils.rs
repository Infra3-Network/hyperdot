use anyhow::anyhow;
use anyhow::Context;
use rust_decimal::prelude::Decimal;
use tokio_postgres::types::FromSql;
use tokio_postgres::types::Type;
use tokio_postgres::Column;
use tokio_postgres::Row;

use crate::types::PostgresColumnData;
use crate::types::PostgresColumnDataType;

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

pub fn f64_to_json_number(val: f64) -> Result<serde_json::Value, anyhow::Error> {
    let temp = serde_json::Number::from_f64(val.into()).ok_or(anyhow!("invalid json-float"))?;
    Ok(serde_json::Value::Number(temp))
}

/// Note. return None should skipped.
pub fn to_json_value(
    row: &Row,
    column: &Column,
    column_idx: usize,
) -> Result<PostgresColumnData, anyhow::Error> {
    // let f64_to_json_number = |raw_val: f64| -> Result<serde_json::Value, anyhow::Error> {
    //     let temp =
    //         serde_json::Number::from_f64(raw_val.into()).ok_or(anyhow!("invalid json-float"))?;
    //     Ok(serde_json::Value::Number(temp))
    // };

    Ok(match *column.type_() {
        // for rust-postgres <> postgres type-mappings: https://docs.rs/postgres/latest/postgres/types/trait.FromSql.html#types
        // for postgres types: https://www.postgresql.org/docs/7.4/datatype.html#DATATYPE-TABLE
        // single types
        Type::BOOL => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: bool| {
                Ok(serde_json::Value::Bool(v))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::BOOL,
                column_value,
            }
        }

        Type::INT2 => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: i16| {
                Ok(serde_json::Value::Number(serde_json::Number::from(v)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::SMALLINT,
                column_value,
            }
        }

        Type::INT4 => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: i32| {
                Ok(serde_json::Value::Number(serde_json::Number::from(v)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::INT,
                column_value,
            }
        }

        Type::INT8 => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: i64| {
                Ok(serde_json::Value::Number(serde_json::Number::from(v)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::BIGINT,
                column_value,
            }
        }

        Type::NUMERIC => {
            let decimal = row
                .try_get::<_, Option<Decimal>>(column_idx)
                .with_context(|| format!("column_name: {}", column.name()))?;

            let column_value = decimal.map_or(serde_json::Value::Null, |decimal| {
                serde_json::Value::String(decimal.to_string())
            });

            PostgresColumnData {
                column_type: PostgresColumnDataType::NUMERIC,
                column_value,
            }
        }

        Type::TEXT => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: String| {
                Ok(serde_json::Value::String(v))
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::TEXT,
                column_value,
            }
        }

        Type::VARCHAR => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: String| {
                Ok(serde_json::Value::String(v))
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::VARCHAR,
                column_value,
            }
        }

        Type::FLOAT4 => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: f32| {
                Ok(f64_to_json_number(v.into())?)
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::FLOAT4,
                column_value,
            }
        }

        Type::FLOAT8 => {
            let column_value = convert_primitive_type(row, column, column_idx, |v: f64| {
                Ok(f64_to_json_number(v)?)
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::FLOAT4,
                column_value,
            }
        }

        // Type::JSON | Type::JSONB => get_basic(row, column, column_i, |a: serde_json::Value| Ok(a))?,
        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR => {
            let column_value =
                convert_primitive_type(row, column, column_idx, |v: StringCollector| {
                    Ok(serde_json::Value::String(v.0))
                })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::TSVECTOR,
                column_value,
            }
        }

        // array types
        Type::BYTEA => {
            let bytea = row
                .try_get::<_, Option<Vec<u8>>>(column_idx)
                .with_context(|| format!("column_name:{}", column.name()))?;

            let column_value = match bytea {
                Some(bytea) => {
                    let mut vala = vec![];
                    for b in bytea {
                        vala.push(serde_json::Value::Number(serde_json::Number::from(b)))
                    }
                    serde_json::Value::Array(vala)
                }
                None => serde_json::Value::Null,
            };
            PostgresColumnData {
                column_type: PostgresColumnDataType::BYTEA,
                column_value,
            }
        }
        Type::BOOL_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: bool| {
                Ok(serde_json::Value::Bool(a))
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::BOOL_ARRAY,
                column_value,
            }
        }
        Type::INT2_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: i16| {
                Ok(serde_json::Value::Number(serde_json::Number::from(a)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::SMALLINT_ARRAY,
                column_value,
            }
        }

        Type::INT4_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: i32| {
                Ok(serde_json::Value::Number(serde_json::Number::from(a)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::INT_ARRAY,
                column_value,
            }
        }

        Type::INT8_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: i64| {
                Ok(serde_json::Value::Number(serde_json::Number::from(a)))
            })?;
            PostgresColumnData {
                column_type: PostgresColumnDataType::BIGINT_ARRAY,
                column_value,
            }
        }

        Type::TEXT_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: String| {
                Ok(serde_json::Value::String(a))
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::TEXT_ARRAY,
                column_value,
            }
        }

        Type::VARBIT_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: String| {
                Ok(serde_json::Value::String(a))
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::VARBIT_ARRAY,
                column_value,
            }
        }

        Type::FLOAT4_ARRAY => {
            let column_value = convert_array_type(row, column, column_idx, |a: f32| {
                Ok(f64_to_json_number(a.into())?)
            })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::FLOAT4_ARRAY,
                column_value,
            }
        }

        Type::FLOAT8_ARRAY => {
            let column_value =
                convert_array_type(row, column, column_idx, |a: f64| Ok(f64_to_json_number(a)?))?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::FLOAT8_ARRAY,
                column_value,
            }
        }

        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR_ARRAY => {
            let column_value =
                convert_array_type(row, column, column_idx, |a: StringCollector| {
                    Ok(serde_json::Value::String(a.0))
                })?;

            PostgresColumnData {
                column_type: PostgresColumnDataType::TSVECTOR_ARRAY,
                column_value,
            }
        }

        Type::JSON_ARRAY | Type::JSONB_ARRAY | Type::JSONB | Type::JSON => {
            unimplemented!("JSON TYPE FAMLIY")
            //    get_array(row, column, column_i, |a: serde_json::Value| Ok(a))?
        }

        _ => anyhow::bail!(
            "Cannot convert pg-cell \"{}\" of type \"{}\" to a serde_json::Value.",
            column.name(),
            column.type_().name()
        ),
    })
}
