# k0r.eu

[![builds.sr.ht status](https://builds.sr.ht/~koehr/k0r/.svg)](https://builds.sr.ht/~koehr/k0r/?)
[![MIT license](https://img.shields.io/badge/license-MIT-blueviolet.svg)](https://opensource.org/licenses/MIT)

A very performant URL shortener service for individuals and small groups.

The service builds upon the [Actix] web framework and [Rusqlite] for data
handling. Thanks to Actix and the speed optimized SQLite database you can
expect 100k requests handled per second on consumer hardware (my laptop).

# Quick Start

The database will be automatically initialized with a super user if it is not yet existing. The api key can be found in the programs output, which should look similar to the following example:

```
$ k0r
Database file k0r.db not found. Create it? [y/N]
y
Added first user with api key 859b397c-a933-461d-a9b1-86dd20084c02
Server is listening on 127.0.0.1:8080
```

This will create a database file in the current directory. You can also give a path instead:

```
$ k0r /path/to/database.db
```

For testing, you can fill the database with test URLs using the application API. There is a helper script and already a file with example URLs inside the `db` folder:

```sh
# assuming you're inside the project directory root and the server is running
./db/insert-via-api.sh 859b397c-a933-461d-a9b1-86dd20084c02 db/test.urls
```

This inserts a bit under two-hundred URLs fetched from [250kb.club](https://git.sr.ht/~koehr/the-250kb-club/tree/main/item/pages.txt) a while ago. The file contains simply one URL per line and the script is not doing any checks and will throw at the API whatever it finds.

# API Usage

Get an URL is straight forward as expected:

```html
$ curl 127.0.0.1:8080/1
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>redirect — k0r link shortener service</title>
  <meta http-equiv="refresh" content="2; url=https://example.com" />
</head>
<body>You will be redirected to <a href="https://example.com">an example</a>.</body>
</html>
```

Inserting a URL is simple as well:

```sh
$ payload='{
    "url":"https://example.com",
    "title":"an example",
    "description":"totally examplary url",
    "key":"859b397c-a933-461d-a9b1-86dd20084c02"
  }'
$ curl -X POST localhost:8080 -H 'Content-Type: application/json' -d $payoad
```

# Planned features

This software is still pre-alpha state and most of the planned features are
not yet implemented. See the [todo list] for more information about the
planned features and current state of implementation.

[Actix]: https://actix.rs/
[Rusqlite]: https://docs.rs/rusqlite/
[250kb.club]: https://git.sr.ht/~koehr/the-250kb-club/tree/main/item/pages.txt
[todo list]: https://todo.sr.ht/~koehr/k0r-planned-features
