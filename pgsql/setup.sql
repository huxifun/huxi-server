create extension if not exists "uuid-ossp";
create extension pg_jieba;

create or replace function set_updated_at()
    returns trigger as
$$
begin
    NEW.updated_at = now();
    return NEW;
end;
$$ language plpgsql;

create or replace function trigger_updated_at(tablename regclass)
    returns void as
$$
begin
    execute format('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD is distinct from NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
end;
$$ language plpgsql;

create collation case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);

-- 文章
CREATE TABLE hu (
  hu_id serial PRIMARY KEY,
  user_id integer NOT NULL,
  user_name text not null,
  i_public smallint not null DEFAULT 0,
  i_type smallint not null,
  i_category smallint not null,
  i_good smallint not null default 0, -- 申请推荐
  title text not null,
  brief text,
  brief_html text,
  body text not null,
  body2 text,
  html text,
  html2 text,
  log text,
  log_html text,
  url text,
  tags text,
  click integer not null DEFAULT 0,
  star integer not null DEFAULT 0, --打星，赞
  good smallint not null DEFAULT 0, --推荐
  created_at  timestamptz not null default now(),
  updated_at timestamptz, 
  good_at timestamptz
);
select trigger_updated_at('hu');

alter table hu add column
  search_ti tsvector GENERATED ALWAYS AS (
      to_tsvector('jiebacfg',
           coalesce(title, '')
           || coalesce(tags, '')
           || coalesce(brief, '')
           || coalesce(body, '')
           || coalesce(body2, '')
      )) STORED;
CREATE INDEX hu_search_idx ON hu USING GIN(search_ti);

--文章评论
CREATE TABLE hu_comment (
  id serial PRIMARY KEY,
  user_id integer not null,
  user_name text not null,
  obj_id integer not null,
  i_public smallint not null DEFAULT 0,
  body text not null,
  html text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
select trigger_updated_at('hu_comment');

-- 微博
CREATE TABLE xi (
  xi_id serial PRIMARY KEY,
  user_id integer NOT NULL,
  user_name text not null,
  i_public smallint not null DEFAULT 0,
  i_type smallint not null default 0,
  i_category smallint not null,
  i_good smallint not null default 0, -- 申请推荐
  title text not null,
  body text not null,
  html text,
  url text,
  tags text,
  click integer not null DEFAULT 0,
  star integer not null DEFAULT 0, --打星，赞
  good smallint not null DEFAULT 0, --推荐
  created_at  timestamptz not null default now(),
  updated_at timestamptz, 
  good_at timestamptz
);

select trigger_updated_at('xi');

alter table xi add column
  search_ti tsvector GENERATED ALWAYS AS (
      to_tsvector('jiebacfg',
           coalesce(title, '')
           || coalesce(tags, '')
           || coalesce(body, '')
      )) STORED;
CREATE INDEX xi_search_idx ON xi USING GIN(search_ti);

--微博评论
CREATE TABLE xi_comment (
  id serial PRIMARY KEY,
  user_id integer not null,
  user_name text not null,
  obj_id integer not null,
  i_public smallint not null DEFAULT 0,
  body text not null,
  html text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
select trigger_updated_at('xi_comment');

--图片
CREATE TABLE image (
  id serial PRIMARY KEY,
  user_id integer NOT NULL,
  title text not null,
  brief text,
  tags text,
  src text,
  file text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
select trigger_updated_at('image');
alter table image add column
  search_ti tsvector GENERATED ALWAYS AS (
      to_tsvector('jiebacfg',
           coalesce(title, '')
           || coalesce(tags, '')
           || coalesce(brief, '')
      )) STORED;
CREATE INDEX image_search_idx ON image USING GIN(search_ti);

--站内短信
CREATE TABLE message (
  id uuid primary key default uuid_generate_v1mc(),
  user_id integer not null,
  user_name text not null,
  to_user_id integer not null,
  to_user_name text not null,
  i_type smallint not null default 0,
  title text not null,
  body text not null,
  html text not null,
  i_status smallint not null default 0,
  in_public smallint not null default 1,
  out_public smallint not null default 1,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);

select trigger_updated_at('message');

--站内短信回复
CREATE TABLE message_comment (
  id serial PRIMARY KEY,
  user_id integer not null,
  user_name text not null,
  message_id integer not null,
  i_public smallint not null DEFAULT 0,
  body text not null,
  html text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
select trigger_updated_at('message_comment');

--密码重置
CREATE TABLE reset_pw_req (
  id uuid primary key default uuid_generate_v1mc(),
  user_id integer NOT NULL,
  user_name text NOT NULL,
  user_email text NOT NULL,
  i_status smallint DEFAULT 0,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);

select trigger_updated_at('reset_pw_req');

--用户
CREATE TABLE users (
  user_id serial PRIMARY KEY,
  uid uuid not null default uuid_generate_v1mc(),
  name text collate "case_insensitive" not null,
  email text collate "case_insensitive" not null,
  password text not null,
  real_name text,
  i_gender smallint,
  birthday timestamptz,
  url text,
  address text,
  corp text,
  mobile text,
  description text,
  image text,
  i_role smallint not null default 0,
  mess_out integer,
  mess_in integer,
  direction text,
  homepage text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
create index on "users" (name collate "ucs_basic");
select trigger_updated_at('users');

--好书
CREATE TABLE book (
  book_id serial PRIMARY KEY,
  user_id integer NOT NULL,
  user_name text not null,
  i_public smallint not null DEFAULT 0,
  i_type smallint not null, -- 格式
  i_category smallint not null,
  i_good smallint not null default 0, -- 申请推荐
  title text not null,
  author text not null,
  brief text,  -- 简介
  brief_html text,
  body text not null, -- 推荐理由
  html text,
  body2 text,  -- 相关链接
  html2 text,
  log text, -- 目录
  log_html text,
  press text, -- 出版社
  version text, -- new
  price text, -- new
  src text, -- 图片源文件 new
  file text, -- 图片文件 new
  url text,
  tags text,
  click integer not null DEFAULT 0,
  star integer not null DEFAULT 0, --打星，赞
  good smallint not null DEFAULT 0, --推荐
  created_at  timestamptz not null default now(),
  updated_at timestamptz, 
  good_at timestamptz
);
select trigger_updated_at('book');

alter table book add column
  search_ti tsvector GENERATED ALWAYS AS (
      to_tsvector('jiebacfg',
           coalesce(title, '')
           || coalesce(tags, '')
           || coalesce(brief, '')
           || coalesce(body, '')
           || coalesce(body2, '')
           || coalesce(log, '')
      )) STORED;
CREATE INDEX book_search_idx ON book USING GIN(search_ti);

--好书评论
CREATE TABLE book_comment (
  id serial PRIMARY KEY,
  user_id integer not null,
  user_name text not null,
  obj_id integer not null,
  i_public smallint not null DEFAULT 0,
  body text not null,
  html text,
  created_at  timestamptz not null default now(),
  updated_at timestamptz
);
select trigger_updated_at('book_comment');
