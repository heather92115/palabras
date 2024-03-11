// @generated automatically by Diesel CLI.

pub mod palabras {
    diesel::table! {
        palabras.awesome_person (id) {
            id -> Int4,
            num_known -> Nullable<Int4>,
            num_correct -> Nullable<Int4>,
            num_incorrect -> Nullable<Int4>,
            total_percentage -> Nullable<Float8>,
            updated -> Timestamptz,
            name -> Nullable<Varchar>,
            code -> Nullable<Varchar>,
            smallest_vocab -> Int4,
        }
    }

    diesel::table! {
        palabras.vocab (id) {
            id -> Int4,
            learning_lang -> Varchar,
            first_lang -> Varchar,
            created -> Timestamptz,
            alternatives -> Nullable<Varchar>,
            skill -> Nullable<Varchar>,
            infinitive -> Nullable<Varchar>,
            pos -> Nullable<Varchar>,
            hint -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        palabras.vocab_study (id) {
            id -> Int4,
            vocab_id -> Int4,
            awesome_person_id -> Int4,
            attempts -> Nullable<Int4>,
            percentage_correct -> Nullable<Float8>,
            last_change -> Nullable<Float8>,
            created -> Timestamptz,
            last_tested -> Nullable<Timestamptz>,
            well_known -> Bool,
            user_notes -> Nullable<Varchar>,
            correct_attempts -> Nullable<Int4>,
        }
    }

    diesel::joinable!(vocab_study -> awesome_person (awesome_person_id));
    diesel::joinable!(vocab_study -> vocab (vocab_id));

    diesel::allow_tables_to_appear_in_same_query!(
        awesome_person,
        vocab,
        vocab_study,
    );
}
