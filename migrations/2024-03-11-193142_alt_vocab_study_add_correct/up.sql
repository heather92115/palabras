alter table vocab_study add column correct_attempts integer DEFAULT 0 check (correct_attempts >= 0);
alter table vocab_study rename column guesses to attempts;