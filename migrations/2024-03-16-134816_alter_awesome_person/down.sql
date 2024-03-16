alter table awesome_person alter column sec_code drop not null;
alter table awesome_person drop constraint sec_code_unique;
alter table awesome_person rename column sec_code to code;
alter table awesome_person drop column max_learning_words;
alter table vocab drop column num_learning_words;
alter table vocab drop column known_lang_code;
alter table vocab drop column learning_lang_code;