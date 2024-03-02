CREATE TABLE progress_stats (
      id SERIAL PRIMARY KEY,
      num_known INTEGER DEFAULT 0 CHECK (num_known >= 0),
      num_correct INTEGER DEFAULT 0 CHECK (num_correct >= 0),
      num_incorrect INTEGER DEFAULT 0 CHECK (num_incorrect >= 0),
      total_percentage FLOAT DEFAULT 0.0 CHECK (total_percentage >= 0.0 AND total_percentage <= 1.0),
      updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO progress_stats (id, total_percentage) VALUES (1, 0.5);