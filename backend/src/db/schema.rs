// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "jobtype"))]
    pub struct Jobtype;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "link"))]
    pub struct Link;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "response"))]
    pub struct Response;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "statuscode"))]
    pub struct Statuscode;
}

diesel::table! {
    _sqlx_migrations (version) {
        version -> Int8,
        description -> Text,
        installed_on -> Timestamptz,
        success -> Bool,
        checksum -> Bytea,
        execution_time -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Jobtype;
    use super::sql_types::Statuscode;
    use super::sql_types::Link;
    use super::sql_types::Response;

    jobs (job_id) {
        job_id -> Text,
        process_id -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Jobtype,
        status -> Statuscode,
        message -> Nullable<Text>,
        created -> Timestamptz,
        finished -> Nullable<Timestamptz>,
        updated -> Timestamptz,
        progress -> Nullable<Int2>,
        links -> Array<Nullable<Link>>,
        response -> Response,
        results -> Nullable<Jsonb>,
        user_id -> Uuid,
    }
}

diesel::table! {
    spatial_ref_sys (srid) {
        srid -> Int4,
        #[max_length = 256]
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        #[max_length = 2048]
        srtext -> Nullable<Varchar>,
        #[max_length = 2048]
        proj4text -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(_sqlx_migrations, jobs, spatial_ref_sys,);
