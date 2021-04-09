create table users
(
    id serial not null
        constraint users_pk
            primary key,
    name text not null
);