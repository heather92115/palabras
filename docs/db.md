
# Setting up your Database

If you cloned/forked this from GitHub I am hoping this won't be overly onerous

_A Postgres Database is required_

## Step 1: Gain admin access to your database.
Install Postgres via Docker or another method
```zsh
docker pull postgres
docker run -d -t -e POSTGRES_PASSWORD="$PGPASSWORD" -v "$DOCKER_VOLUMES"/pg/data -p 5432:5432 postgres
docker run --name test_postgres -d -t -e POSTGRES_PASSWORD="$TEST_PGPASSWORD" -v "$DOCKER_VOLUMES"/pg2/data -p 5433:5432 postgres
```

## Step 2: Create the schema: "palabras"

To create the schema, run the following SQL
```sql
CREATE USER developer WITH PASSWORD '****';

CREATE SCHEMA palabras;
ALTER DATABASE postgres SET search_path TO palabras;

GRANT ALL PRIVILEGES ON SCHEMA palabras TO developer;
ALTER ROLE developer SET search_path TO palabras;
```
_Note: there is a lot of variance allowed here other than the 
name of the schema needs to be 'palabras' and the user needs to be able to 
alter the schema._

## Step 3: Create a .env file with the DB URL
```zsh
echo "export PALABRA_DATABASE_URL='postgres://developer:************@0.0.0.0/palabras'" > .env
```
_Note: Don't source the env files if you plan to switch between prod and testing. The command line shell's env vars will override
the loaded .env file values._

# For Integration Tested, repeat the previous steps with slight alterations:

## Step 1: Gain admin access to your database.

## Step 2: Create the schema: "palabras"
```sql
CREATE USER tester WITH PASSWORD '';

CREATE SCHEMA palabras;
ALTER DATABASE postgres SET search_path TO palabras;

GRANT ALL PRIVILEGES ON SCHEMA palabras TO tester;
ALTER ROLE tester SET search_path TO palabras;
```

## Step 3: Create a test.env file with the DB URL
```zsh
echo "export PALABRA_DATABASE_URL='postgres://tester:************@0.0.0.0:5433/palabras'" > testing.env
```
_Note: Don't source the env files if you plan to switch between prod and testing. The command line shell's env vars will override
the loaded .env file values._

# Install Postges via Docker or another method
```zsh
docker pull postgres
docker run -d -t -e POSTGRES_PASSWORD="$PGPASSWORD" -v "$DOCKER_VOLUMES"/pg/data -p 5432:5432 postgres
docker run --name test_postgres -d -t -e POSTGRES_PASSWORD="$TEST_PGPASSWORD" -v "$DOCKER_VOLUMES"/pg2/data -p 5433:5432 postgres
```
