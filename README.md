# Huxi Server

Huxi Server is a site engine for web applications, based on Axum and PostgreSQL. 

The document in English is being written, and the following is in Chinese.

Demo: [http://www.huxi.fun](http://www.huxi.fun)

## 介绍

Huxi(呼吸) Server 是用Rust编写的网站系统，主要用于构建网站应用。

## 已实现功能

* 用户注册登录
* 电子邮件验证
* 博客文章
* 微博
* 好书
* 图片
* 站内信箱


## 基本架构

* Web 框架使用 `axum`
* 页面模板使用 `maud`
* 前端CSS使用 `bootstrap`
* 数据库使用 `PostgreSQL` 和 `sqlx` 
* 全文检索使用 [pg_jieba](https://github.com/jaiminpan/pg_jieba)

## 安装

### 准备数据库 

1. 安装 PostgreSQL
2. 安装 [pg_jieba](https://github.com/jaiminpan/pg_jieba)
3. 新建数据库 `www`，导入 `pgsql/setup.sql`，创建表
```
psql www < pgsql/setup.sql
```

详细说明见 [pgsql/README.md](pgsql/README.md)。

### 栏目基本配置

```
cp examples/config.toml my-config.toml
```

编辑 `my-config.toml`，其中`SMTP`设置用于用户注册。

### 设置环境变量

示例:

```bash
export WWW_CONFIG=/home/huxi/has/my-config.toml
export WWW_PORT=3000
export DATABASE_URL=postgres://huxi:12345678@localhost/www
```

## 运行 

```
cargo run
```

打开网址： http://localhost:3000

## 管理员

注册用户后，在pgsql中，修改 users.i_role = 5, 例如：

```sql
update users set i_role=5 where name='admin';
```

## Nginx https 部署

见 `examples/nginx.conf`

## TODO

* 完善栏目管理
* 完善用户管理
* 优化页面

## 联系作者

川月（huxifun@sina.com）
