FROM postgres:latest AS pg

COPY schema.sql userinit.sql /docker-entrypoint-initdb.d/
