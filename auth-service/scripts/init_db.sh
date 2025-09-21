DB_PATH=./.local.pg
DB_LOG_PATH=./.local.pg.log
DB_NAME=local
DB_USER=rusty

pg_ctl stop -D $DB_PATH

if [ -f .local.pg ]; then
    echo 'Local Postgres data folder exists, starting...'
    pg_ctl start -D $DB_PATH -l $DB_LOG_PATH
else
    echo 'Initialising local database...'
    pg_ctl init -D $DB_PATH -o "-E UTF8 -U $DB_USER -A trust"

    echo 'Starting local database...'
    pg_ctl start -D $DB_PATH -l $DB_LOG_PATH

    echo 'Creating local database...'
    createdb -h localhost -p 5432 -U $DB_USER $DB_NAME
fi
