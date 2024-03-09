use async_graphql::*;
use crate::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};


#[derive(Clone)]
struct Challenge {
    vocab_id: i32,
    vocab_study_id: i32,
    prompt: String,
}

#[Object]
impl Challenge {

    async fn vocab_id(&self) -> i32 {
        self.vocab_id
    }

    async fn vocab_study_id(&self) -> i32 {
        self.vocab_study_id
    }

    async fn prompt(&self) -> String {
        self.prompt.clone()
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Returns the sum of a and b
    async fn study(&self, awesome_id: i32, limit: i64) -> Result<Vec<Challenge>> {

        let match_service = VocabFuzzyMatch::instance();

        let mut study_list: Vec<Challenge> = Vec::new();

        let vocab = match_service.get_vocab_to_learn(awesome_id, limit)?;
        for (vs, v) in vocab {
            let prompt = match_service.determine_prompt(&v, &vs.user_notes.unwrap_or_default());
            study_list.push(Challenge {
                vocab_id: v.id,
                vocab_study_id: vs.id,
                prompt
            });
        }

        Ok(study_list)
    }
}

pub struct StudyMutation;

#[Object]
impl StudyMutation {
    async fn check_response(&self, vocab_id: i32,vocab_study_id: i32,  entered: String) -> Result<String> {

        let match_service = VocabFuzzyMatch::instance();

        let prompt =
            match_service.check_response(
                vocab_id,
                vocab_study_id,
                entered)?;

        Ok(prompt)
    }
}