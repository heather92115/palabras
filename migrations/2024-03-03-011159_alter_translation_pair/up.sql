alter table translation_pair drop column direction;
alter table translation_pair add column hint varchar DEFAULT '';
