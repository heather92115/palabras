alter table vocab_study drop column correct_attempts;
alter table vocab_study rename column attempts to guesses;
