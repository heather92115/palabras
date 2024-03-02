CREATE TABLE translation_pair (
          id SERIAL PRIMARY KEY,
          learning_lang VARCHAR NOT NULL UNIQUE,
          first_lang VARCHAR NOT NULL,
          percentage_correct FLOAT DEFAULT 0.0 CHECK (percentage_correct >= 0.0 AND percentage_correct <= 1.0),
          created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
          last_tested TIMESTAMP WITH TIME ZONE,
          fully_known BOOLEAN NOT NULL DEFAULT FALSE
);
