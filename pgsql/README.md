# PostgreSQL 常用命令

以下示例基于 `Ubuntu 20.04 LTS` 系统。

编译安装 [pg_jieba](https://github.com/jaiminpan/pg_jieba)。

安装好 pg_jieba 后，可以在 `/usr/lib/postgresql/12/lib/` 中看到 `pg_jieba.so`。

别忘了在 `/etc/postgresql/12/main/postgresql.conf` 中加入以下内容：

```
default_text_search_config = 'jiebacfg'
shared_preload_libraries = 'pg_jieba.so'	# (change requires restart)
```

重启 postgresql

```
sudo systemctl restart postgresql
```

切换到用户 postgres

```bash
sudo -iu postgres
# or
su -l postgres
pgsql
```

创建用户 huxi 和 数据库 www

```sql
create user huxi with password '123456';
alter user huxi with superuser;
create database www;
grant all privileges on database www to huxi;
```

导入sql文件

```
psql www < pgsql/setup.sql
# or
psql -d www -U huxi -f pgsql/setup.sql
```

导出sql文件

```
pg_dump -f backup.sql www
```
