# Configuration
There are 3 main sources of configuration (for both the server and the cli), in order of priority:
1. The command line
2. Environment variables
3. A config file (TOML)

Both the server and the cli share the same config file.
The options are documented below.

## The CLI Configuration (`[cli]`)
You don't have to specify the database url or the media directory each time you invoke the cli: put these in the config file.

The keys are documented below.

### cli.db-url
Env: `BLOG_CLI_DB_URL`\
Type: string

This key specifies the database connection string used in the blog cli.
It must start with `postgres://` or `postgresql://`.

### cli.media-dir
Env: `BLOG_CLI_MEDIA_DIR`\
Type: path (string)

This key specifies the media directory which the media uploads will be copied to.

### cli.editor
Env: `BLOG_CLI_EDITOR`\
Type: [command](#the-command-type)

The editor field specifies what command to run to launch the text editor while editing articles, notes and more.
If not specified, the `EDITOR` environment variable will be used or a system default editor will be launched.

### cli.hooks
> This is a table and has no environment or cli configuration. It is available in the config file.

The `cli.hooks` table stores various hooks for the blog cli.

#### cli.hooks.pre-db
Type: [command](#the-command-type)

The hook to run before attempting to connect to the database.

#### cli.hooks.pre-media
Type: [command](#the-command-type)

The hook to run before attempting any media operation, including uploading, deleting or renaming files.

#### cli.hooks.post-media
Type: [command](#the-command-type)

The hook to run after all the media operations are done, before the command exits.

## The Server Configuration (`[server]`)
All server configuration can be done through the config file or by setting environment variables.

### server.site-name
Env: `BLOG_SERVER_SITE_NAME`\
Type: string

The name of the site. This will be used in page titles and must not include HTML.

### server.description
Env: `BLOG_SERVER_DESCRIPTION`\
Type: string

A brief description of the site. This is used in the `<meta description...>` tag included in every page. Keep it brief and do nto use HTML.

### server.db-url
The database connection string for the web server.
It must start with `postgres://` or `postgresql://`.

### server.listen
Env: `BLOG_SERVER_LISTEN`\
Type: string

The interface address the server will serve on. E.g `0.0.0.0:80`

### server.media-dir
Env: `BLOG_SERVER_MEDIA_DIR`\
Type: path (string)

The media directory to be served on `/media`.
It defaults to `$PWD/media`.

### server.copyright-holder
Env: `BLOG_SERVER_COPYRIGHT_HOLDER`\
Type: string

The name of the copyright holder.
This will be used in the site footer.

### server.site-url
Env: `BLOG_SERVER_SITE_URL`\
Type: string

The full URL, including the protocol, of the website.
The Atom feed will be disabled if this field is not set.
Example: `https://www.example.com`

# The command Type
In some places, a configuration keys value is of the `command` type.
This translates to a bash style command string or an array of strings representing a command and its arguments.
For example, both below represent the same command:

- `foo = "bash -c 'echo hello world'"`
- `foo = ["bash", "-c", "hello world"]`

Empty strings (`""`) and empty arrays (`[]`) are errors.

If specified through environment variables, values of this type can only be strings in the syntax above.

# An Example Config File
```toml
# You would normally configure the cli section on your local machine
# and configure the server section in the production server.
[cli]
db-url = "postgres://admin:my_password@localhost:3001/blog"
media-dir = "/home/blog/media"
editor = "nano"

[server]
description = "A blog about music and programming"
listen = "0.0.0.0:8080"
media-dir = "/home/blog/media"
copyright-holder = "John Doe"
site-url = "http://localhost:8080"
site-name = "Example Blog"

[cli.hooks]
pre-db = "echo 'connecting to the database!'"
pre-media = ["echo", "doing a media operation..."]
```

Of course, this is a demonstrative configuration; you would normally not have your cli configuration on the same machine as your server.
