# k0r.eu

[![builds.sr.ht status](https://builds.sr.ht/~koehr/k0r/.svg)](https://builds.sr.ht/~koehr/k0r/?)
[![MIT license](https://img.shields.io/badge/license-MIT-blueviolet.svg)](https://opensource.org/licenses/MIT)

A URL shortener for individuals, but optimized for speed. GET requests to an URL typically takes less than 100µs.

Quick Start
-----------

Initialize a fresh SQlite database:

```
sqlite3 /path/to/k0r.db < ./db/schema.sql
```

For testing, you can use `test_urls.sql` for test data:

```
# after initialization
sqlite3 /path/to/k0r.db < ./db/test_urls.sql
```

This inserts a bit under two-hundred URLs fetched from [250kb.club](https://git.sr.ht/~koehr/the-250kb-club/tree/main/item/pages.txt) a while ago.

To start the service, just run the binary from the folder of `k0r.db` or run it with the path to the DB as first argument. For example:

```
k0r /path/to/ # will be extended to /path/to/k0r.db
k0r /path/to/some.db
```
