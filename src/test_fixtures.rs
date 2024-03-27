#[cfg(test)]
use crate::dal::awesome_person::AwesomePersonRepository;
use crate::dal::vocab::VocabRepository;
use crate::dal::vocab_study::VocabStudyRepository;
use crate::models::NewVocabStudy;
use crate::models::{AwesomePerson, NewAwesomePerson, NewVocab, Vocab, VocabStudy};
use crate::sl::fuzzy_match_vocab::VocabFuzzyMatch;

pub struct TestFixtures {
    pub fuzzy_service: Box<VocabFuzzyMatch>,
}

// Create a mocked fuzzy service for unit tests. Repos are mocked
// and return test data
pub fn fixture_setup() -> TestFixtures {
    let awesome_person_repo = Box::new(MockAwesomePersonRepository);

    let (vocab_study, vocab_study_list, vocab, vocab_list, combo_list) = create_test_data();

    let vocab_study_repo = Box::new(MockVocabStudyRepository {
        vocab_study,
        vocab_study_list,
        combo_list,
    });

    let vocab_repo = Box::new(MockVocabRepository { vocab, vocab_list });

    let fuzzy_service = Box::new(VocabFuzzyMatch::new(
        awesome_person_repo,
        vocab_study_repo,
        vocab_repo,
    ));

    TestFixtures { fuzzy_service }
}

fn create_test_data() -> (
    VocabStudy,
    Vec<VocabStudy>,
    Vocab,
    Vec<Vocab>,
    Vec<(VocabStudy, Vocab)>,
) {
    let vocab_study = VocabStudy {
        id: 1,
        vocab_id: 1,
        awesome_person_id: 1,
        attempts: Some(1),
        percentage_correct: Some(0.99),
        last_change: None,
        created: Default::default(),
        last_tested: None,
        well_known: true,
        user_notes: None,
        correct_attempts: None,
    };

    let vocab_study_list = vec![vocab_study.clone()];

    let vocab = Vocab {
        id: 1,
        learning_lang: "palabra".to_string(),
        first_lang: "word".to_string(),
        created: Default::default(),
        alternatives: None,
        skill: None,
        infinitive: None,
        pos: Some("noun".to_string()),
        hint: None,
        num_learning_words: 1,
        known_lang_code: "en".to_string(),
        learning_lang_code: "es".to_string(),
    };

    let vocab_list = vec![vocab.clone()];

    let combo_list = vec![(vocab_study.clone(), vocab.clone())];

    (vocab_study, vocab_study_list, vocab, vocab_list, combo_list)
}

// Mock-up functions to simulate actual function behaviors
pub struct MockAwesomePersonRepository;

impl AwesomePersonRepository for MockAwesomePersonRepository {
    fn get_awesome_person_by_id(&self, stats_id: i32) -> Result<Option<AwesomePerson>, String> {
        Ok(Some(AwesomePerson {
            id: stats_id,
            num_known: Some(100),
            num_correct: Some(80),
            num_incorrect: Some(20),
            total_percentage: Some(0.8),
            updated: chrono::Utc::now(),
            name: None,
            sec_code: "3456".to_string(),
            smallest_vocab: 5,
            max_learning_words: 5,
        }))
    }

    fn get_awesome_person_by_code(
        &self,
        lookup_code: String,
    ) -> Result<Option<AwesomePerson>, String> {
        Ok(Some(AwesomePerson {
            id: 23,
            num_known: Some(200),
            num_correct: Some(180),
            num_incorrect: Some(20),
            total_percentage: Some(0.9),
            updated: chrono::Utc::now(),
            name: None,
            sec_code: lookup_code,
            smallest_vocab: 2,
            max_learning_words: 5,
        }))
    }

    fn update_awesome_person(&self, _stats: AwesomePerson) -> Result<usize, String> {
        Ok(1)
    }

    fn create_awesome_person(
        &self,
        new_awesome_person: &NewAwesomePerson,
    ) -> Result<AwesomePerson, String> {
        Ok(AwesomePerson {
            id: 2,
            num_known: new_awesome_person.num_known,
            num_correct: new_awesome_person.num_correct,
            num_incorrect: new_awesome_person.num_incorrect,
            total_percentage: new_awesome_person.total_percentage,
            name: new_awesome_person.name.clone(),
            sec_code: "fsfd-df9a".to_string(),
            ..Default::default()
        })
    }
}

// Mock struct for VocabStudyRepository
pub struct MockVocabStudyRepository {
    pub vocab_study: VocabStudy,
    pub vocab_study_list: Vec<VocabStudy>,
    pub combo_list: Vec<(VocabStudy, Vocab)>,
}

// Mock implementation of VocabRepository
impl VocabStudyRepository for MockVocabStudyRepository {
    fn get_vocab_study_by_id(&self, vocab_id: i32) -> Result<VocabStudy, String> {
        // Mock behavior: returns our previously setup test data
        Ok(VocabStudy {
            id: vocab_id,
            ..self.vocab_study.clone()
        })
    }

    fn get_vocab_study_by_foreign_refs(
        &self,
        vocab_id: i32,
        awesome_person_id: i32,
    ) -> Result<Option<VocabStudy>, String> {
        // Mock behavior: Return an Ok result
        Ok(Some(VocabStudy {
            vocab_id,
            awesome_person_id,
            ..self.vocab_study.clone()
        }))
    }

    fn get_study_set(
        &self,
        _awesome_person_id: i32,
        _max_words_in_phrase: i32,
    ) -> Result<Vec<(VocabStudy, Vocab)>, String> {
        Ok(self.combo_list.clone()) // returns our test data from mem
    }

    fn create_vocab_study(&self, new_vocab_study: &NewVocabStudy) -> Result<VocabStudy, String> {
        let vocab_study = VocabStudy {
            id: 2,
            vocab_id: new_vocab_study.vocab_id.clone(),
            awesome_person_id: new_vocab_study.awesome_person_id.clone(),
            ..self.vocab_study.clone()
        };

        Ok(vocab_study)
    }

    fn update_vocab_study(&self, _updating: VocabStudy) -> Result<usize, String> {
        Ok(1)
    }
}

// Mock struct for VocabRepository
pub struct MockVocabRepository {
    pub vocab: Vocab,
    pub vocab_list: Vec<Vocab>,
}

// Mock implementation of VocabRepository
impl VocabRepository for MockVocabRepository {
    fn get_vocab_by_id(&self, vocab_id: i32) -> Result<Vocab, String> {
        Ok(Vocab {
            id: vocab_id,
            ..self.vocab.clone()
        })
    }

    fn find_vocab_by_learning_language(
        &self,
        learning_lang_search: String,
    ) -> Result<Option<Vocab>, String> {
        Ok(Some(Vocab {
            learning_lang: learning_lang_search,
            ..self.vocab.clone()
        }))
    }

    fn find_vocab_by_alternative(
        &self,
        alternative_search: String,
    ) -> Result<Option<Vocab>, String> {
        Ok(Some(Vocab {
            alternatives: Some(alternative_search),
            ..self.vocab.clone()
        }))
    }

    fn get_empty_first_lang(&self, _limit: i64) -> Result<Vec<Vocab>, String> {
        Ok(vec![Vocab {
            first_lang: "".to_string(),
            ..self.vocab.clone()
        }])
    }

    fn create_vocab(&self, new_vocab: &NewVocab) -> Result<Vocab, String> {
        let vocab = Vocab {
            learning_lang: new_vocab.learning_lang.clone(),
            first_lang: new_vocab.first_lang.clone(),
            alternatives: new_vocab.alternatives.clone(),
            ..self.vocab.clone()
        };

        Ok(vocab)
    }

    fn update_vocab(&self, _updating: Vocab) -> Result<usize, String> {
        Ok(1)
    }
}
