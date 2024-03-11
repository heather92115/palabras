use crate::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};
use async_graphql::*;

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

#[derive(Clone)]
struct AwesomeProfile {
    id: i32,               // Primary key used to look up stats in the data layer.
    num_known: i32,        // Number of pairs moved to fully known state.
    num_correct: i32,      // Total number of correct guesses.
    num_incorrect: i32,    // Total number of incorrect guesses.
    total_percentage: f64, // Percentage guess correctly.
    name: String,          // User's name, optional
    smallest_vocab: i32,   // Size of the smallest vocab word to be tested.
}

#[Object]
impl AwesomeProfile {
    async fn id(&self) -> i32 {
        self.id
    }
    async fn num_known(&self) -> i32 {
        self.num_known
    }
    async fn num_correct(&self) -> i32 {
        self.num_correct
    }
    async fn num_incorrect(&self) -> i32 {
        self.num_incorrect
    }
    async fn total_percentage(&self) -> f64 {
        self.total_percentage
    }
    async fn name(&self) -> String {
        self.name.clone()
    }
    async fn smallest_vocab(&self) -> i32 {
        self.smallest_vocab
    }
}

#[derive(Clone)]
struct VocabStats {
    learning: String,        // The word being studied.
    attempts: i32,           // The number of times this vocab was attempted.
    correct_attempts: i32,   // The number of times this vocab was correctly attempted.
    percentage_correct: f64, // The percentage of correct guesses calculated using the distance from the correct match.
    last_change: f64,        // The most recent percentage correct change
    last_tested: String,     // The last time this pair was attempted.
}

#[Object]
impl VocabStats {

    async fn learning(&self) -> String {
        self.learning.clone()
    }

    async fn attempts(&self) -> i32 {
        self.attempts
    }

    async fn correct_attempts(&self) -> i32 {
        self.correct_attempts
    }

    async fn percentage_correct(&self) -> f64 {
        self.percentage_correct
    }

    async fn last_change(&self) -> f64 {
        self.last_change
    }

    async fn last_tested(&self) -> String {
        self.last_tested.clone()
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
                prompt,
            });
        }

        Ok(study_list)
    }

    async fn get_awesome_person(&self, awesome_id: i32) -> Result<AwesomeProfile> {
        let match_service = VocabFuzzyMatch::instance();
        let pub_awesome_person = match_service.get_awesome_person(awesome_id)?;
        let pub_awesome_person = pub_awesome_person.unwrap_or_default();

        Ok(AwesomeProfile {
            id: pub_awesome_person.id,
            num_known: pub_awesome_person.num_known.unwrap_or_default(),
            num_correct: pub_awesome_person.num_correct.unwrap_or_default(),
            num_incorrect: pub_awesome_person.num_incorrect.unwrap_or_default(),
            total_percentage: pub_awesome_person.total_percentage.unwrap_or_default(),
            name: pub_awesome_person.name.unwrap_or_default(),
            smallest_vocab: pub_awesome_person.smallest_vocab,
        })
    }

    async fn get_vocab_stats(&self, vocab_study_id: i32) -> Result<VocabStats> {

        let match_service = VocabFuzzyMatch::instance();

        let (vocab_study, vocab) = match_service.get_vocab_stats(vocab_study_id)?;

        let last_tested = if vocab_study.last_tested.is_some() {
            vocab_study.last_tested.unwrap().format("%Y-%m-%d %H:%M:%S %Z").to_string()
        } else {
            "".to_string()
        };

        Ok(VocabStats {
            learning: vocab.learning_lang.clone(),
            attempts: vocab_study.attempts.unwrap_or_default(),
            correct_attempts: vocab_study.correct_attempts.unwrap_or_default(),
            percentage_correct: vocab_study.percentage_correct.unwrap_or_default(),
            last_change: vocab_study.last_change.unwrap_or_default(),
            last_tested
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn check_response(
        &self,
        vocab_id: i32,
        vocab_study_id: i32,
        entered: String,
    ) -> Result<String> {
        let match_service = VocabFuzzyMatch::instance();

        let prompt = match_service.check_response(vocab_id, vocab_study_id, entered)?;

        Ok(prompt)
    }
}
