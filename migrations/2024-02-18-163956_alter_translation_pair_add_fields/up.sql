
alter table translation_pair add column skill varchar DEFAULT '';
alter table translation_pair add column too_easy boolean DEFAULT false not null;