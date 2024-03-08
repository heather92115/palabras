
#[cfg(test)]
use crate::dal::awesome_person::AwesomePersonRepository;
use crate::dal::vocab_study::VocabStudyRepository;
use crate::models::{AwesomePerson, Vocab, VocabStudy};
use crate::models::{NewVocabStudy};
use diesel::result::Error as DieselError;
use crate::sl::study_vocab::VocabFuzzyMatch;

pub struct TestFixtures {
    pub fuzzy_service: Box<VocabFuzzyMatch>
}

pub fn fixture_setup() -> TestFixtures {
    let awesome_person_repo = Box::new(MockAwesomePersonRepository);
    let vocab_study_repo = Box::new(MockVocabStudyRepository);

    let fuzzy_service = Box::new(VocabFuzzyMatch::new(
        awesome_person_repo,
        vocab_study_repo,
    ));

    TestFixtures { fuzzy_service }
}


// Mock-up functions to simulate actual function behaviors
pub struct MockAwesomePersonRepository;

impl AwesomePersonRepository for MockAwesomePersonRepository {
    fn get_awesome_person_by_id(&self, stats_id: i32) -> Result<AwesomePerson, DieselError> {
        Ok(AwesomePerson {
            id: stats_id,
            num_known: Some(100),
            num_correct: Some(80),
            num_incorrect: Some(20),
            total_percentage: Some(0.8),
            updated: chrono::Utc::now(),
            name: None,
            code: None,
            smallest_vocab: 0,
        })
    }

    fn update_awesome_person(&self, _stats: AwesomePerson) -> Result<usize, String> {
        Ok(1)
    }
}

// Mock struct for VocabRepository
pub struct MockVocabStudyRepository;

// Mock implementation of VocabRepository
impl VocabStudyRepository for MockVocabStudyRepository {
    fn get_vocab_study_by_id(&self, vocab_id: i32) -> Result<VocabStudy, DieselError> {
        // Mock behavior: Return an Ok result with a dummy VocabStudy
        Ok(VocabStudy {
            id: vocab_id,
            percentage_correct: Some(0.5),
            ..Default::default()
        })
    }

    fn get_vocab_study_by_foreign_refs(&self, vocab_id: i32, awesome_person_id: i32) -> Result<Option<VocabStudy>, DieselError> {
        // Mock behavior: Return an Ok result
        Ok(Some(VocabStudy {
            id: 2, // Simulate that a new ID was assigned
            vocab_id,
            awesome_person_id,
            percentage_correct: Some(0.5),
            ..Default::default()
        }))
    }

    fn get_study_set(&self, _awesome_person_id: i32) -> Result<Vec<(VocabStudy, Vocab)>, String> {
        todo!()
    }

    fn create_vocab_study(
        &self,
        _new_vocab_study: &NewVocabStudy,
    ) -> Result<VocabStudy, String> {
        // Mock behavior: Return an Ok result with a newly "created" Vocab
        Ok(VocabStudy {
            id: 2, // Simulate that a new ID was assigned
            vocab_id: 2,
            awesome_person_id: 1,
            percentage_correct: Some(0.5),
            ..Default::default()
        })
    }

    fn update_vocab_study(&self, _updating: VocabStudy) -> Result<usize, String> {
        // Mock behavior: Return Ok(1) to simulate a successful update
        Ok(1)
    }
}
