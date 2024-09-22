# Examples

There are a number of examples and project template repositories, which can be
used as a starting point for new projects.

**Examples**

  - [Full example](examples/full) with all features enabled
  - [Minimal example](examples/minimal) with only the core features enabled
  - [API example](examples/api) suitable for building APIs

**Templates**

  - [Simple, single-crate project template](https://github.com/danwilliams/terracotta-template-simple)
  - [Multi-crate workspace project template](https://github.com/danwilliams/terracotta-template-workspace)

This document provides information about how to configure the "full" example,
which has all features enabled.

The main sections in this document are:

  - [Configuration](#configuration)
  - [Running](#running)


## Configuration

A Terracotta-based project is typically configured using a TOML file. The
default configuration file is `Config.toml`, which should be placed in the
repository root. The configuration settings (and file) are optional, and if not
provided, default values will be used for all configuration options.

It is also possible to pass configuration parameters from the command line, as
environment variables. The environment variables take precedence over the
configuration file options.

### General options

The following options should be specified without any heading:

  - `host`   - The host to listen on. Defaults to `127.0.0.1`.
  - `port`   - The port to listen on. Defaults to `8000`.
  - `logdir` - The directory to store log files in. Defaults to `log`.
  - `title`  - The title of the application. Defaults to `Terracotta`.

As shown here:

```toml
host   = "127.0.0.1"
port   = 8000
logdir = "log"
title  = "Terracotta"
```

### Local loading options

By default, all resources are baked into the binary, and served from there. This
is the most efficient way to run the application, but it is also possible to
load resources from the local filesystem, which can be useful for development
and testing, and when there are large content files.

It is possible to supplement or override HTML templates and static assets.
Static assets are subdivided into protected and public.

The following type headings are available:

  - `html_templates`   - HTML templates.
  - `assets.protected` - Protected static assets.
  - `assets.public`    - Public static assets.

The following options should be specified under the individual type headings:

  - `behavior`   - The loading behaviour.
  - `local_path` - The path to the files on the local filesystem.

The `behavior` option can be one of the following values:

  - `Deny`       - Deny loading from the local filesystem. This is the default
                   for all the options.
  - `Supplement` - Load from the local filesystem if the baked-in resources are
                   not present.
  - `Override`   - Load from the local filesystem if present, and otherwise load
                   from the baked-in resources.

For those types configured to allow loading from the local filesystem, the
following options can be specified under the individual type headings:

  - `local_path` - The path to the files.

As shown here:

```toml
[html_templates]
behavior   = "Deny"
local_path = "html"

[assets.protected]
behavior   = "Override"   # default is "Deny"
local_path = "content"

[assets.public]
behavior   = "Override"   # default is "Deny"
local_path = "static"
```

An example is provided, `rustacean-flat-happy.png`, which is available through
http://localhost:8000/rustacean-flat-happy.png if using the settings in the
example configuration file. This is a protected asset, and so will only be
served to logged-in users.

### Static file options

When static files are requested, the method by which they are served depends
upon their source and size. All files baked into the binary are served directly
from memory, and so these options do not apply to them. Files loaded from the
local filesystem are loaded into memory and served all once if they are small
enough, but past a certain (configurable) size they are streamed to the client.

The sizes of the stream buffer and read buffer are hugely important to
performance, with smaller buffers greatly impacting download speeds. The default
values have been carefully chosen based on extensive testing, and should not
generally need to be changed. However, on a system with lots of users and very
few large files it *may* be worth decreasing the buffer sizes to reduce memory
usage when those files are requested, and on a system with very few users and
lots of large files it *may* be worth increasing the buffer sizes to improve
throughput. However, the chosen values are already within 5-10% of the very best
possible speeds, so any increase should be made with caution. It is more likely
that they would need to be decreased a little on a very busy system with a lot
of large files, where the memory usage could become a problem and the raw speed
of each download becomes a secondary concern.

The following options should be specified under an `[assets.static_files]`
heading:

  - `stream_threshold` - The size of the file, in KB, above which it will be
                         streamed to the client. Defaults to `1000` (1MiB).
  - `stream_buffer`    - The size of the stream buffer to use when streaming
                         files, in KB. Defaults to `256` (256KB).
  - `read_buffer`      - The size of the read buffer to use when streaming
                         files, in KB. Defaults to `128` (128KB).

Each of these options accepts an integer value.

As shown here:

```toml
[assets.static_files]
stream_threshold = 1000 # 1MiB — files above this size will be streamed
stream_buffer    = 256  # 256KB
read_buffer      = 128  # 128KB
```

### User list

A list of user credentials can be specified under a `[users]` heading:

  - `username: password` - The username as the key, and the password as the
                           value.

As shown here:

```toml
[users]
joe = "1a2b3c"
```

This is a simple list of username/password pairs, where the username is the key
and the password is the value. The password is stored in plain text, so be aware
of the security implications of this (ideally you would implement an integration
with your preferred database instead). The username and password are both
case-sensitive.

## Running

The examples can be run using the `cargo run` command, and passing the required
features:

```sh
cargo run --example full --features=full
cargo run --example minimal --features="errors health tera"
cargo run --example api --features="errors health stats utoipa"
```

The example application server will listen on port 8000 by default, and will
serve content from the `static` directory, plus any request handlers that you
define. The `static` directory contains the static files to be served.


