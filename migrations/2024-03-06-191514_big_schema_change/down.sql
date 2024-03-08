
alter table vocab
    add column percentage_correct float default 0.0 check (percentage_correct >= 0.0 and percentage_correct <= 1.0),
    add column fully_known boolean not null default false,
    add column guesses integer default 0 check (guesses >= 0),
    add column too_easy boolean not null default false,
    add column last_tested timestamp with time zone,
    add column user_notes varchar default '';

-- Pull fields back into the vocab table.
update vocab set guesses = vocab_study.guesses,
                 percentage_correct = vocab_study.percentage_correct,
                 last_tested = vocab_study.last_tested,
                 fully_known = vocab_study.well_known,
                 user_notes = vocab_study.user_notes
from (select
          vocab_id, guesses, percentage_correct, last_tested, well_known, user_notes
      from vocab_study) as vocab_study
where vocab_study.vocab_id = vocab.id;

drop table vocab_study;

alter table vocab rename to translation_pair;

alter table awesome_person
    drop column smallest_vocab,
    drop column code,
    drop column name;

alter table awesome_person rename to progress_stats;

