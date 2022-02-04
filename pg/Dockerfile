FROM postgres:14.1 AS pg

COPY schema.sql userinit.sql /docker-entrypoint-initdb.d/
