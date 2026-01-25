-- Add migration script here
create table if not exists users
(
    id integer primary key not null,
    username text not null unique,
    password text not null
);
