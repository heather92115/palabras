alter table awesome_person rename column code to sec_code;
alter table awesome_person alter column sec_code set not null;
alter table awesome_person add constraint sec_code_unique unique (sec_code);
alter table awesome_person add column max_learning_words integer DEFAULT 5 check (max_learning_words >= 1) not null;
alter table vocab add column num_learning_words integer DEFAULT 1 check (num_learning_words >= 1) not null;
alter table vocab add column known_lang_code varchar DEFAULT 'en' not null;
alter table vocab add column learning_lang_code varchar DEFAULT 'es' not null;

UPDATE vocab v
SET num_learning_words = subquery.word_count
FROM (
         SELECT v1.id, COUNT(*) AS word_count
         FROM vocab v1, LATERAL regexp_split_to_table(v1.learning_lang, '\s+') AS words(word)
         GROUP BY v1.id
     ) AS subquery
WHERE v.id = subquery.id;
