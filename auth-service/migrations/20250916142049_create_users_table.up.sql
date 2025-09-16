create table if not exists users(
    email text not null primary key,
    password_hash text not null,
    requires_2fa boolean not null default false
);
