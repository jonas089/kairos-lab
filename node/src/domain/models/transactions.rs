use chrono::NaiveDateTime;
use serde::Serialize;
use diesel::deserialize::FromSql;
use deadpool_diesel::postgres::Pool;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;
use std::io::Write;

use crate::database::schema;
use crate::database::schema::transactions;
use crate::database::errors;

#[derive(Serialize, Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = schema::sql_types::Trxtype)]
pub enum TrxType {
    Transfer,
    Withdrawal,
    Deposit,
}


impl ToSql<schema::sql_types::Trxtype, Pg> for TrxType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            TrxType::Transfer => out.write_all(b"Transfer")?,
            TrxType::Withdrawal => out.write_all(b"Withdrawal")?,
            TrxType::Deposit => out.write_all(b"Deposit")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<schema::sql_types::Trxtype, Pg> for TrxType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Transfer" => Ok(TrxType::Transfer),
            b"Withdrawal" => Ok(TrxType::Withdrawal),
            b"Deposit" => Ok(TrxType::Deposit),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// Define struct for schema for transfers
#[derive(Insertable, Queryable, Debug, PartialEq, Selectable)]
#[diesel(table_name = schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub timestamp: NaiveDateTime,
    pub public_key: String,
    pub nonce: i64,
    pub trx: TrxType,
    pub amount: i64,
    pub recipient: Option<String>,
}

pub struct TransactionInput {
    pub public_key: String,
    pub nonce: i64,
    pub trx: TrxType,
    pub amount: i64,
    pub recipient: Option<String>,
}

// impl To<Transaction> for TransactionInput {
//     Transaction {
//         timestamp: Utc::now().NaiveDateTime
//         public_key,
//         nonce,
//         trx,
//         amount,
//         recipient,
//     }
// }

pub struct TransactionFilter {
    public_key: Option<String>,
}

pub async fn insert(pool: Pool, transaction: Transaction) -> Result<Transaction, errors::DatabaseError> {
    let conn = pool.get().await?;
    let res = conn
            .interact(|conn| {
                diesel::insert_into(transactions::table)
                    .values(transaction)
                    .get_result::<Transaction>(conn)
            })
        .await??;
    Ok(res)
}

pub async fn get_all(pool: Pool, filter: TransactionFilter) -> Result<Vec<Transaction>, errors::DatabaseError> {
    let conn = pool.get().await?;
    let res = conn
        .interact(move |conn| {
            let mut query = transactions::table.into_boxed::<diesel::pg::Pg>();

            if let Some(public_key) = filter.public_key {
                query = query.filter(transactions::public_key.eq(public_key));
            }

            query.select(Transaction::as_select()).load::<Transaction>(conn)
        })
        .await??;
    Ok(res)
}