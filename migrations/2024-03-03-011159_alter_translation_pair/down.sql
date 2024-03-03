alter table translation_pair add column direction varchar DEFAULT '';
alter table translation_pair drop column hint;
