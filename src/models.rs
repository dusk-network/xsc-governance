use async_graphql::*;
use rusqlite::{
    types::{FromSql, FromSqlError, ValueRef},
    Result, Row,
};
use tai64::Tai64;

#[derive(SimpleObject, Debug)]
pub struct Transfer {
    to: [u8; 8],
    from: [u8; 8],
    amount: u64,
    block_height: u64,
    timestamp: Timestamp,
}

#[derive(Debug)]
pub struct Timestamp(Tai64);

#[Scalar]
impl ScalarType for Timestamp {
    fn parse(value: Value) -> Result<Self, InputValueError<Self>> {
        if let Value::Number(number) = value {
            if let Some(num_64) = number.as_u64() {
                Ok(Timestamp(Tai64(num_64)))
            } else {
                Err(InputValueError::expected_type(Value::Number(Number::from(
                    0_u64,
                ))))
            }
        } else {
            Err(InputValueError::expected_type(Value::Number(Number::from(
                0_u64,
            ))))
        }
    }

    fn to_value(&self) -> Value {
        let tai = self.0;

        Value::Number(Number::from(tai.0))
    }
}

impl From<u64> for Timestamp {
    fn from(val: u64) -> Self {
        Timestamp(Tai64(val))
    }
}

impl FromSql for Timestamp {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        let i = u64::column_result(value)?;
        Ok(Timestamp::from(i))
    }
}

impl TryFrom<&Row<'_>> for Transfer {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self> {
        Ok(Transfer {
            to: row.get(2)?,
            from: row.get(3)?,
            amount: row.get(4)?,
            block_height: row.get(5)?,
            timestamp: row.get(6)?,
        })
    }
}
