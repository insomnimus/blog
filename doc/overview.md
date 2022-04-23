# App Overview
The app consists of 3 parts:
- A PostgreSql database
- A web server
- A command line content management utility

These are briefly described below.

## The Database
Text content published to the site is stored in the database.
Media uploads are not stored in the database; they are stored on a filesystem which the server has direct access to.
Media uploads are recorded in the database for the server to recognize them (this also makes cleaning up garbage possible).

It is not enforced by code but it's strongly recommended to have a read-only user for the web server.

## The Web Server
> For more information, visit [the configuration page](configuration.md).

The web server is a standalone program that simply consumes the database content.
In fact, it's just some routers and a cache layer with a database connection.

On startup the configuration will be read from a config file.
Every configurable aspect of the web server can be done through the command line as well.

The web server requires the `static` directory, along with the CSS and JavaScript files in it to be present.
These files are included in the git repository.

Another required directory is the `media` directory.
The web server will serve it on the `/media` endpoint. This is not strictly necessary if you do not plan to upload any media.
However you still need to provide a valid directory since this feature can't be turned off.

## The Content Management Utility
The crud operations are done through the `blog` command line utility.
The usage of the utility is documented on a separate page: [The blog CLI](cli.md).

Just like how the web server communicates with the database directly, the blog cli accesses the database for crud operations without going through any proxy.

To make this convenient without sacrificing security, the cli supports various hooks to be executed before and after database/media access.
For example, one might want to establish an ssh reverse proxy to a remote server that hosts the database before connecting to the database.

The cli is responsible for rendering markdown to html (both the source and the rendered form are stored in the database for future edits to the source).
The reasoning behind this is, the translation from markdown to html has to be done only once.
If the web server were to do it, it would be sensible to store the rendered document back in the database so as to not re-render after a reboot.
This would mean that the web server would be writing to the database, which both requires an extra permission and is a bad design to follow.

Media uploads are done through similar mechanisms except the "uploads" are actually just local filesystem copies.
To actually host the site on a separate device, managing it remotely, pre-media and post-media hooks are offered.

For example, one might mount a remote filesystem over [sshfs](https://en.wikipedia.org/wiki/SSHFS) before any media operation is done (through the pre-media hook)
and use the post-media hook to unmount the same sshfs mount.
