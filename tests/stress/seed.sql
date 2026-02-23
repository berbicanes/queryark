-- Stress test seed: 5 schemas x 400 tables x 10 columns = 2000 tables
-- Used to validate sidebar performance with large schemas

DO $$
DECLARE
  schema_names TEXT[] := ARRAY['inventory', 'analytics', 'customers', 'orders', 'logistics'];
  s TEXT;
  i INT;
  j INT;
  ddl TEXT;
  idx_ddl TEXT;
  fk_ddl TEXT;
BEGIN
  FOREACH s IN ARRAY schema_names LOOP
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', s);

    FOR i IN 1..400 LOOP
      ddl := format(
        'CREATE TABLE %I.table_%s (
          id SERIAL PRIMARY KEY,
          col_text_1 TEXT,
          col_text_2 VARCHAR(255),
          col_int_1 INTEGER DEFAULT 0,
          col_int_2 BIGINT,
          col_bool BOOLEAN DEFAULT false,
          col_ts TIMESTAMPTZ DEFAULT now(),
          col_numeric NUMERIC(12,2),
          col_json JSONB,
          col_uuid UUID DEFAULT gen_random_uuid()
        )', s, lpad(i::TEXT, 4, '0'));
      EXECUTE ddl;

      -- Add an index on col_text_1
      idx_ddl := format(
        'CREATE INDEX idx_%s_table_%s_text1 ON %I.table_%s (col_text_1)',
        s, lpad(i::TEXT, 4, '0'), s, lpad(i::TEXT, 4, '0'));
      EXECUTE idx_ddl;

      -- Add a composite index on (col_int_1, col_ts)
      idx_ddl := format(
        'CREATE INDEX idx_%s_table_%s_int1_ts ON %I.table_%s (col_int_1, col_ts)',
        s, lpad(i::TEXT, 4, '0'), s, lpad(i::TEXT, 4, '0'));
      EXECUTE idx_ddl;

      -- Add FK from every 10th table to the previous table in the same schema
      IF i > 1 AND i % 10 = 0 THEN
        fk_ddl := format(
          'ALTER TABLE %I.table_%s ADD CONSTRAINT fk_%s_%s_to_%s
           FOREIGN KEY (col_int_1) REFERENCES %I.table_%s(id)',
          s, lpad(i::TEXT, 4, '0'),
          s, lpad(i::TEXT, 4, '0'), lpad((i-1)::TEXT, 4, '0'),
          s, lpad((i-1)::TEXT, 4, '0'));
        EXECUTE fk_ddl;
      END IF;
    END LOOP;

    -- Insert a few rows into the first 10 tables so stats queries return data
    FOR i IN 1..10 LOOP
      FOR j IN 1..100 LOOP
        EXECUTE format(
          'INSERT INTO %I.table_%s (col_text_1, col_text_2, col_int_1, col_bool, col_numeric)
           VALUES (%L, %L, %s, %s, %s)',
          s, lpad(i::TEXT, 4, '0'),
          'text_' || j, 'value_' || j, j, j % 2 = 0, j * 1.5);
      END LOOP;
    END LOOP;
  END LOOP;

  -- Create some views in the first schema
  FOR i IN 1..20 LOOP
    EXECUTE format(
      'CREATE VIEW inventory.view_%s AS SELECT id, col_text_1, col_int_1 FROM inventory.table_%s',
      lpad(i::TEXT, 4, '0'), lpad(i::TEXT, 4, '0'));
  END LOOP;

  -- Create some functions
  FOR i IN 1..10 LOOP
    EXECUTE format(
      'CREATE OR REPLACE FUNCTION inventory.fn_%s(p_id INTEGER)
       RETURNS TEXT LANGUAGE sql AS $fn$
         SELECT col_text_1 FROM inventory.table_%s WHERE id = p_id
       $fn$',
      lpad(i::TEXT, 4, '0'), lpad(i::TEXT, 4, '0'));
  END LOOP;

  -- Create some sequences
  FOR i IN 1..5 LOOP
    EXECUTE format('CREATE SEQUENCE inventory.seq_%s START 1', lpad(i::TEXT, 4, '0'));
  END LOOP;

  -- Create some enum types
  EXECUTE 'CREATE TYPE inventory.status_type AS ENUM (''active'', ''inactive'', ''pending'', ''archived'')';
  EXECUTE 'CREATE TYPE inventory.priority_type AS ENUM (''low'', ''medium'', ''high'', ''critical'')';

  RAISE NOTICE 'Stress test seed complete: 5 schemas x 400 tables = 2000 tables';
END $$;
