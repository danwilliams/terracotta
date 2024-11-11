# Terracotta

![Rust](https://img.shields.io/badge/Rust-1.81%2B-b7410e?style=flat&logo=rust&logoColor=white&labelColor=b7410e)
[![Crate version](https://img.shields.io/crates/v/terracotta?style=flat)](https://crates.io/crates/terracotta)
[![CI](https://img.shields.io/github/actions/workflow/status/danwilliams/terracotta/ci.yml?style=flat&logo=github&logoColor=white&label=build%2Ftest)](https://github.com/danwilliams/terracotta/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/docsrs/terracotta?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/crate/terracotta/latest)
![License](https://img.shields.io/github/license/danwilliams/terracotta?style=flat)

[Axum]:       https://crates.io/crates/axum
[Terracotta]: https://crates.io/crates/terracotta
[Tokio]:      https://crates.io/crates/tokio

[Terracotta][] is a Rust library for building web applications and APIs, based
on [Axum][] and [Tokio][].

The name Terracotta was chosen because it's rusty in colour, and clay represents
something that can be moulded into different shapes.

There are a number of [examples and project template repositories](docs/examples.md),
which can be used as a starting point for new projects.

**Examples**

  - [Full example](examples/full) with all features enabled
  - [Minimal example](examples/minimal) with only the core features enabled
  - [API example](examples/api) suitable for building APIs

**Templates**

  - [Simple, single-crate project template](https://github.com/danwilliams/terracotta-template-simple)
  - [Multi-crate workspace project template](https://github.com/danwilliams/terracotta-template-workspace)

Terracotta, and the project templates based on it, are intended to be easy to
use and understand, easy to set up and extend, and easy to deploy.

Terracotta was created in response to the lack of full examples of how to use
Axum, and the fact that many tutorials are out-of-date, lacking important
elements, or just plain wrong. You may not need everything provided — and you
also may well not agree with how some parts are implemented — but if you are
wanting a leg-up to save some time, it's not a bad place to start!

The main sections in this README are:

  - [Features](#features)
  - [Usage](#usage)
  - [Attributions](#attributions)

Additional documentation of note includes:

  - [Developer documentation](docs/developer.md)
  - [Examples documentation](docs/examples.md)
  - [API Integration documentation](docs/integration.md)


## Features

[Bulma]:        https://bulma.io/
[Figment]:      https://crates.io/crates/figment
[Font Awesome]: https://fontawesome.com/
[Tera]:         https://crates.io/crates/tera
[Tracing]:      https://crates.io/crates/tracing
[Hyper]:        https://crates.io/crates/hyper

The features referred to here are partly that of the Terracotta library itself,
and partly that of the example applications that are provided (through the
[examples and template project repositories](docs/examples.md)).

The main high-level points of note are:

  - Simple codebase layout
  - Full yet minimal web application working out of the box
  - Easy to extend and build upon
  - High-performance asynchronous HTTP server using [Tokio Hyper][Hyper]
  - Based on the robust and ergonomic web framework [Axum][]
  - Configuration from config file and env vars using [Figment][]
  - Logging of HTTP requests and events using [Tokio Tracing][Tracing]
  - Templates implemented using the [Tera][] template engine
  - Static file handling
  - Ability to serve static files as protected or public
  - Ability to supplement and override the static assets using local files in
    addition to a pre-compiled binary (configurable)
  - Streaming of large static files for memory efficiency
  - Single-file deployment — all assets baked in (optional and configurable)
  - CSS foundation using the [Bulma][] CSS framework
  - Icons using [Font Awesome][]
  - Simple authentication using sessions and config-based user list
  - Login page, public and protected routes, logout ability
  - Health check API endpoints
  - Comprehensive application statistics gathering and API endpoints for
    reporting
  - Graceful handling of 404 and 500 HTTP errors
  - Graceful handling of runtime application errors
  - Full OpenAPI documentation

### Authentication

Terracotta features a custom-rolled authentication system, to demonstrate how to
implement a basic session-based setup. Although storing a user list in plain
text on a server is okay for small, limited projects and experiments, it is
highly recommended to store the credentials securely in a database. That is
currently outside the scope of this project, for a number of reasons.

In a real system you will probably also want to store the sessions in a database
instead of in memory.

It is also worth noting that the pattern implemented is the best and most ideal
for an application that serves HTML to a browser. If you are creating an API
then you will want to change some of the behaviour to return HTTP status codes
to tell the client that the request is unauthorised, rather than sending a login
page. Equally, you will likely want to implement JWT or similar. That is also
outside of scope at present, partly because there are various options to choose
from. However, there is a [basic API example](examples/api/) available.

The authentication system is set up to make it easy to configure routes as
either public or protected, and is fully-implemented including a login page,
logout action, and handling of every part of the authentication journey and the
possible situations.

### Statistics

Terracotta gathers a wide range of statistics about the application, and
provides a number of API endpoints to access them. These are intended to be used
by monitoring systems, and also to provide a simple way to see what is going on
with the application.

The approach is highly performant, despite gathering a comprehensive set of
measurements, and is designed to be as efficient as possible. It is also
designed to be easy to extend, to add new statistics to the system. There is a
central statistics queue plus broadcast system, with circular buffers for
interval history, and a tick clock to keep everything up-to-date.

The statistics data is available in summary form, per-measurement history form,
and as a real-time WebSocket event stream.

### Error-handling

Terracotta has an opinionated approach to handling errors, including both HTTP
errors and "true" (Rust) errors. This serves as a baseline to build on or to
change as required.

### Databases

Terracotta very purposefully does not include any kind of database integration.
There are so many, and such a plethora of crates to choose from, that this is
best left to the application developer to decide. Database interaction is very
straightforward and so this is a simple addition to make.

### Templates

The choice of Tera is unlikely to upset anyone, but if there is a preferred
option then it is easy to change or remove. Tera has been implemented in a
slightly opinionated manner, but it should be clear what has been changed from
the defaults and how.


## Usage

The Terracotta project template repositories are designed so that they can be
used as starting points for new projects, and then customised and extended.

Terracotta itself is a library, as available on [crates.io][Terracotta]. This
means that most of the functionality is abstracted into this crate repository,
and not in the template repositories.

  - For information on developing a project based on Terracotta, please refer to
    the [Developer documentation](docs/developer.md).

  - For information about running and using the examples, please refer to the
    [Examples documentation](docs/examples.md).


## Attributions

[Bulma license]:        https://github.com/jgthms/bulma/blob/master/LICENSE
[CC-BY license]:        https://creativecommons.org/licenses/by/4.0/
[Font Awesome license]: https://fontawesome.com/license/free
[MIT license]:          http://opensource.org/licenses/MIT
[Public Domain]:        https://creativecommons.org/publicdomain/zero/1.0/
[Rust logo use]:        https://github.com/rust-lang/rust/issues/11562#issuecomment-50833809
[Rust logo]:            https://github.com/rust-lang/rust/issues/11562#issuecomment-32700278
[Rustacean]:            https://rustacean.net/
[SIL OFL license]:      https://scripts.sil.org/OFL

The following attributions are made for the use of third-party content in the
examples (and also for the project template repositories made available).

Terracotta uses the [Rust logo][] as a default for its examples, due to being
written in Rust. The logo is [freely usable][Rust logo use] under the [CC-BY
(Creative Commons Attribution) license][CC-BY license].

An image of Ferris the crab (the Rust mascot) is used as an example of protected
content. This image is sourced from [rustacean.net][Rustacean] and is in the
[Public Domain][], so can be freely used.

The [Bulma CSS framework][Bulma] is [published][Bulma license] under the [MIT
license][] and free to use without restriction.

The [Font Awesome][] icons are [published][Font Awesome license] under the
[CC-BY (Creative Commons Attribution) license][CC-BY license], and the webfonts
under the [SIL OFL (Open Font License)][SIL OFL license]. They are freely
usable, along with the CSS code used to display them, which is released under
the [MIT license][].


