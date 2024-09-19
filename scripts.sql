--create table
-- Table: public.tasks

-- DROP TABLE IF EXISTS public.tasks;

CREATE TABLE IF NOT EXISTS public.tasks
(
    name character varying COLLATE pg_catalog."default",
    schedule character varying COLLATE pg_catalog."default",
    url character varying COLLATE pg_catalog."default",
    "executiontimeout" bigint,
    "retrytimes" bigint,
    "retryinterval" bigint,
    "retryjitterlimit" bigint
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.tasks
    OWNER to postgres;