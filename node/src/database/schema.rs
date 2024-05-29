// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "trxtype"))]
    pub struct Trxtype;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Trxtype;

    transactions (timestamp, amount, public_key) {
        timestamp -> Timestamp,
        public_key -> Varchar,
        nonce -> Int8,
        trx -> Trxtype,
        amount -> Int8,
        recipient -> Nullable<Varchar>,
    }
}
