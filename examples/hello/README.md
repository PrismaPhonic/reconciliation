# Hello Controller

This folder contains a hello-world example demonstrating how to build your own
reconciliation controller.

First, make sure that you set an env var of `DATABASE_URL` to point at your full
mysql connection string, including the database in the format:

```
export DATABASE_URL=mysql://user:pass@host:port/dbname
```

Ensure that you have imported the schema inside of `schema.sql` in this
directory into that database.

Then, simply run this example with `cargo run --example hello`.

If we insert our name into the hello table:

```sql

INSERT INTO hello (name, created_at, updated_at) VALUES ('peter', now(), now());
```

We should shortly see a message pop up in the `hello_status` table, written by
the hello controller.

```sql
MySQL [test]> SELECT * FROM hello_status;
+----+----------+----------------------------+----------------------------+------------+---------------+
| id | hello_id | created_at                 | updated_at                 | deleted_at | message       |
+----+----------+----------------------------+----------------------------+------------+---------------+
|  1 |        1 | 2021-10-02 04:55:13.252629 | 2021-10-02 04:55:13.252631 | NULL       | Hello, peter! |
+----+----------+----------------------------+----------------------------+------------+---------------+
1 row in set (0.000 sec)
```

We can also soft delete the row and see the reconciler automatically pick up
deleting it, as well as any associated rows in hello_status:

```sql
MySQL [test]> UPDATE hello SET deleted_at = now() WHERE id = 1;
Query OK, 1 row affected (0.020 sec)
Rows matched: 1  Changed: 1  Warnings: 0

MySQL [test]> SELECT * FROM hello;
Empty set (0.000 sec)

MySQL [test]> SELECT * FROM hello_status;
Empty set (0.000 sec)
```

Enjoy!
