use super::schema::{jobs, sql_types};
use chrono::{DateTime, Utc};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{Output, ToSql, WriteTuple},
    sql_types::{BigInt, Nullable, SqlType, Text},
};
use diesel_derive_enum::DbEnum;
use o2o::o2o;
use serde::Deserialize;

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob<'a> {
    pub job_id: &'a str,
    pub process_id: Option<&'a str>,
    pub status: StatusCode,
    pub message: Option<&'a str>,
    pub job_type: JobType,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub progress: Option<i16>,
    pub links: Vec<Link>,
    pub response: Response,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = jobs)]
pub struct UpdateJob<'a> {
    pub status: StatusCode,
    pub message: Option<&'a str>,
    pub updated: DateTime<Utc>,
    pub progress: Option<i16>,
    pub links: Vec<Link>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = jobs)]
pub struct UpdateJobStatus<'a> {
    pub status: StatusCode,
    pub message: Option<&'a str>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = jobs)]
pub struct FinishJob<'a> {
    pub status: StatusCode,
    pub message: Option<&'a str>,
    pub updated: DateTime<Utc>,
    pub finished: DateTime<Utc>,
    pub progress: Option<i16>,
    pub links: Vec<Link>,
    pub results: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = jobs)]
pub struct DismissJob<'a> {
    pub status: StatusCode,
    pub message: Option<&'a str>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize, HasQuery, o2o)]
#[owned_into(ogcapi::types::processes::StatusInfo)]
#[diesel(table_name = jobs)]
pub struct StatusInfo {
    pub job_id: String,
    pub process_id: Option<String>,
    #[map(~.into())]
    pub status: StatusCode,
    pub message: Option<String>,
    #[map(r#type, ~.into())]
    pub job_type: JobType,
    #[map(~.into())]
    pub created: DateTime<Utc>,
    #[map(~.into())]
    pub updated: DateTime<Utc>,
    pub finished: Option<DateTime<Utc>>,
    #[map(~.map(|p| p as u8))]
    pub progress: Option<i16>,
    #[map(~.into_iter().filter_map(|l| l.map(Into::into)).collect())]
    pub links: Vec<Option<Link>>,
}

#[derive(Debug, Deserialize, DbEnum, SqlType, o2o)]
#[from_owned(ogcapi::types::processes::JobType)]
#[owned_into(ogcapi::types::processes::JobType)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::JobType")]
pub enum JobType {
    Process,
}

#[derive(Debug, Deserialize, DbEnum, SqlType, o2o)]
#[from_owned(ogcapi::types::processes::StatusCode)]
#[owned_into(ogcapi::types::processes::StatusCode)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::StatusCode")]
pub enum StatusCode {
    Accepted,
    Running,
    Successful,
    Failed,
    Dismissed,
}

#[derive(Debug, Deserialize, AsExpression, FromSqlRow, o2o)]
#[from_owned(ogcapi::types::common::Link)]
#[owned_into(ogcapi::types::common::Link)]
#[ghosts(templated: None, var_base: None)]
#[diesel(sql_type = sql_types::Link, postgres_type(name = "Link"))]
pub struct Link {
    pub href: String,
    pub rel: String,
    pub r#type: Option<String>,
    pub hreflang: Option<String>,
    pub title: Option<String>,
    pub length: Option<i64>,
}

impl ToSql<sql_types::Link, Pg> for Link {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        // Write the fields in the order: href TEXT, rel TEXT, type TEXT, hreflang TEXT, title TEXT, length BIGINT
        WriteTuple::<(
            Text,
            Text,
            Nullable<Text>,
            Nullable<Text>,
            Nullable<Text>,
            Nullable<BigInt>,
        )>::write_tuple(
            &(
                &self.href,
                &self.rel,
                self.r#type.as_ref(),
                self.hreflang.as_ref(),
                self.title.as_ref(),
                self.length.as_ref(),
            ),
            &mut out.reborrow(),
        )
    }
}

impl FromSql<sql_types::Link, Pg> for Link {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        // Use the tuple implementation to extract the fields
        let (href, rel, r#type, hreflang, title, length): (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
        ) = <(
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
        ) as FromSql<
            diesel::sql_types::Record<(
                Text,
                Text,
                Nullable<Text>,
                Nullable<Text>,
                Nullable<Text>,
                Nullable<BigInt>,
            )>,
            Pg,
        >>::from_sql(bytes)?;

        Ok(Link {
            href,
            rel,
            r#type,
            hreflang,
            title,
            length,
        })
    }
}

#[derive(Debug, Deserialize, SqlType, DbEnum, o2o)]
#[from_owned(ogcapi::types::processes::Response)]
#[owned_into(ogcapi::types::processes::Response)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::Response")]
pub enum Response {
    Raw,
    Document,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_jobtype_from_string() {
        let v: JobType = serde_json::from_str("\"Process\"").expect("to deserialize JobType");
        assert!(matches!(v, JobType::Process));
    }

    #[test]
    fn deserialize_statuscode_variants() {
        let s = serde_json::from_str::<StatusCode>("\"Accepted\"").expect("accepted");
        assert!(matches!(s, StatusCode::Accepted));

        let s = serde_json::from_str::<StatusCode>("\"Running\"").expect("running");
        assert!(matches!(s, StatusCode::Running));

        let s = serde_json::from_str::<StatusCode>("\"Successful\"").expect("successful");
        assert!(matches!(s, StatusCode::Successful));

        let s = serde_json::from_str::<StatusCode>("\"Failed\"").expect("failed");
        assert!(matches!(s, StatusCode::Failed));

        let s = serde_json::from_str::<StatusCode>("\"Dismissed\"").expect("dismissed");
        assert!(matches!(s, StatusCode::Dismissed));
    }

    #[test]
    fn deserialize_response_enum() {
        let r: Response = serde_json::from_str("\"Raw\"").expect("raw");
        assert!(matches!(r, Response::Raw));
    }
}
