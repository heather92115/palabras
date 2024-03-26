

# There are some strategies to employ. Google for translations from your learning language to your first language (or whatever known language you are using).

## Just enter data directly onto the database.
```sql
UPDATE palabras.translation_pair SET first_lang = 'place' WHERE id = 1;
```
## Other Importing Strategies

1. Export your word pairs with missing first language translations.
```zsh
cargo run --bin export_missing_first data/my_export.csv
```
_Note: The export file will default to data/export.csv_

2. Obtain the translations back to your first language using AI

I had limited success with `ChatGpt4` with the prompt below. It would only translate roughly 30 rows for me.

```text
Translate each row to English, using appropriate pronouns (e.g., 'I', 'you', 'they'). Format the output as a CSV with 'Learning' and 'English' columns.

learning, infinitive, pos
aprenden,aprender,Verb
trabaje,trabajar,Verb
salgo,salir,Verb
estudia,estudiar,Verb
trabajas,trabajar,Verb
```
If you are willing to pay for OpenAI tokens try using their [playground](https://platform.openai.com/playground/p/Jogp3Rnx4OLET8khBW5BUHDy?mode=chat).

I used Model: `gpt-4-turbo-preview` with Maximum length: `4095`. It translated 300 rows  accurately enough for my purposes. I think you could translate a larger set successfully.

3. Less costly ideas include Googling for lists of translations. You will need to get the translations into a data file.
https://strommeninc.com/1000-most-common-spanish-words-frequency-vocabulary/
https://github.com/mananoreboton/en-es-en-Dic/blob/master/src/main/resources/dic/es-en.xml

4. Update [translations_config.json](../translations_config.json) to look for your translation files. Care must be taken to look for the correct columns.

