

alter table translation_pair rename to vocab;

alter table progress_stats rename to awesome_person;

alter table awesome_person
    add column name varchar DEFAULT '',
    add column code varchar DEFAULT '',
    add column smallest_vocab int not null DEFAULT 1  check (smallest_vocab >= 1);

create table vocab_study (
              id serial primary key,
              vocab_id int not null,
              awesome_person_id int not null,
              guesses integer default 0 check (guesses >= 0),
              percentage_correct float default 0.0 check (percentage_correct >= 0.0 and percentage_correct <= 1.0),
              last_change float default 0.0,
              created timestamp with time zone not null default now(),
              last_tested timestamp with time zone,
              well_known boolean not null default false,
              user_notes varchar default '',
              unique(vocab_id, awesome_person_id),
              constraint fk_vocab_study_vocab
                foreign key (vocab_id) references vocab(id),
              constraint fk_vocab_study_awesome_person
                foreign key (awesome_person_id) references awesome_person(id)
);

-- Copy progress fields back to the new vocab_study table
insert into vocab_study (vocab_id, awesome_person_id, guesses, percentage_correct, last_tested, well_known, user_notes)
select id, 1, guesses, percentage_correct, last_tested, fully_known, user_notes
from vocab;

-- Now the old vocab columns can be dropped
alter table vocab
    drop column percentage_correct,
    drop column fully_known,
    drop column guesses,
    drop column too_easy,
    drop column last_tested,
    drop column user_notes;

