table! {
    benchmark (id) {
        id -> Uuid,
        title -> Text,
        subject -> Text,
        difficulty -> Text,
        creator_id -> Nullable<Uuid>,
        git_url -> Nullable<Text>,
        max_cyclomatic_complex -> Int4,
    }
}

table! {
    submission (id) {
        id -> Uuid,
        language -> Text,
        code -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        user_id -> Uuid,
        status -> Text,
        benchmark_id -> Nullable<Uuid>,
        stdout -> Nullable<Text>,
        stderr -> Nullable<Text>,
        exec_duration -> Int4,
        message -> Nullable<Text>,
        error -> Nullable<Text>,
        lint_score -> Nullable<Int4>,
        quality_score -> Nullable<Int4>,
        mem_usage -> Int4,
        code_hash -> Nullable<Text>,
        cyclomatic_complexity -> Int4,
    }
}

table! {
    user (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        password -> Text,
        username -> Text,
        data_version -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(
    benchmark,
    submission,
    user,
);
