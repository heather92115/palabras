use std::vec::IntoIter;
use crate::models::TranslationPair;
use crate::sl::learn_pairs::{create_fuzzy_match_service, LearnTranslationPairs};

pub trait ManageStudySet {
    fn next_study_set(&mut self, num_words: i64);

    fn check_pair_distance(&mut self, user_response: &String) -> String;

    fn next(&mut self);

    fn determine_word_prompt(&self) -> String;

    fn determine_match_prompt(&self, user_response: &String) -> String;

    fn remaining_study_pairs(&self) -> i64;

    fn has_vocab_ready(&self) -> bool;
}
pub struct StudySet {
    match_service: Box<dyn LearnTranslationPairs>,
    study_set: IntoIter<TranslationPair>,
    match_distance: usize,
    current_vocab: Option<TranslationPair>,
    remaining: i64
}

impl Default for StudySet {
    fn default() -> Self {
        Self {
            match_service: create_fuzzy_match_service(),
            study_set: Default::default(),
            match_distance: 0,
            current_vocab: None,
            remaining: 0,
        }
    }
}

impl ManageStudySet for StudySet {
    fn next_study_set(&mut self, num_words: i64) {
        if let Ok(study_list) = self.match_service.get_study_pairs(num_words) {
            self.remaining = study_list.len() as i64;
            self.study_set = study_list.into_iter();
            self.current_vocab = self.study_set.next();
        }
    }

    fn check_pair_distance(&mut self, user_response: &String)  -> String {

        if self.current_vocab.is_some() && !user_response.is_empty() {
            let tp = self.current_vocab.clone().unwrap_or_default();

            self.match_distance = self.match_service.check_pair_match(&tp.learning_lang,
                                                &tp.alternatives.clone().unwrap_or_default(),
                                                user_response);
            _ = self.match_service.update_pair_stats(tp.clone().id, self.match_distance);

            self.remaining = self.remaining - 1;

            return self.determine_match_prompt(user_response);
        }

        "".to_string()
    }

    fn next(&mut self) {
        self.current_vocab = self.study_set.next();
    }

    fn determine_word_prompt(&self) -> String {

        if self.current_vocab.clone().is_some() {
            self.match_service.determine_prompt(self.current_vocab.clone().unwrap())
        } else {
            "You're all done for now!".to_string()
        }
    }

    fn determine_match_prompt(&self, user_response: &String) -> String {

        let tp = self.current_vocab.clone().unwrap_or_default();

        return if self.match_distance == 0 {
            "Perfect Match!".to_string()
        } else if self.match_distance <= 3 {
            format!("Close, it was '{}', you entered '{}'", tp.learning_lang, user_response)
        } else {
            format!("It was '{}', you entered '{}'", tp.learning_lang, user_response)
        };
    }

    fn remaining_study_pairs(&self) -> i64 {
        self.remaining
    }

    fn has_vocab_ready(&self) -> bool {
        self.current_vocab.clone().is_some()
    }
}

