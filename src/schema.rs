// @generated automatically by Diesel CLI.

pub mod palabras {
    diesel::table! {
        palabras.progress_stats (id) {
            id -> Int4,
            num_known -> Nullable<Int4>,
            num_correct -> Nullable<Int4>,
            num_incorrect -> Nullable<Int4>,
            total_percentage -> Nullable<Float8>,
            updated -> Timestamptz,
        }
    }

    diesel::table! {
        palabras.translation_pair (id) {
            id -> Int4,
            learning_lang -> Varchar,
            first_lang -> Varchar,
            percentage_correct -> Nullable<Float8>,
            created -> Timestamptz,
            last_tested -> Nullable<Timestamptz>,
            fully_known -> Bool,
            guesses -> Nullable<Int4>,
            alternatives -> Nullable<Varchar>,
            skill -> Nullable<Varchar>,
            too_easy -> Bool,
            infinitive -> Nullable<Varchar>,
            pos -> Nullable<Varchar>,
            direction -> Nullable<Varchar>,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(progress_stats, translation_pair,);
}
