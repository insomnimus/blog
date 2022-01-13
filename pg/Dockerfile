FROM postgres:latest AS pg

COPY init.sql schema.sql /docker-entrypoint-initdb.d/
