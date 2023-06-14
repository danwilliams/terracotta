# Terracotta

This repository provides a boilerplate webserver application, based on [Axum](https://crates.io/crates/axum),
to act as a foundation and springboard for building web applications and APIs.

The name Terracotta was chosen because it's rusty in colour, and clay represents
something that can be moulded into different shapes.

It is intended to be easy to use and understand, easy to fork and extend, and
easy to deploy.

Terracotta was created in response to the lack of full examples of how to use
Axum, and the fact that many tutorials are out-of-date, lacking important
elements, or just plain wrong. You may not need everything provided - and you
also may well not agree with how some parts are implemented - but if you are
wanting a leg-up to save some time, it's not a bad place to start!

The main sections in this README are:

  - [Features](#features)
  - [Setup](#setup)
  - [Usage](#usage)
  - [Deployment](#deployment)
  - [Attributions](#attributions)


## Features

The main high-level points of note are:

  - Simple codebase layout
  - Full yet minimal web application working out of the box
  - Easy to extend and build upon
  - High-performance asynchronous HTTP server using [Tokio Hyper](https://crates.io/crates/hyper)
  - Based on the robust and ergonomic web framework [Axum](https://crates.io/crates/axum)
  - Configuration from config file and env vars using [Figment](https://crates.io/crates/figment)
  - Logging of HTTP requests and events using [Tokio Tracing](https://crates.io/crates/tracing)
  - Templates implemented using the [Tera](https://crates.io/crates/tera)
    template engine
  - Static file handling
  - Single-file deployment - all assets baked in
  - CSS foundation using the [Bulma](https://bulma.io/) CSS framework
  - Icons using [Font Awesome](https://fontawesome.com/)
  - Simple authentication using sessions and config-based user list
  - Login page, public and protected routes, logout ability
  - Graceful handling of 404 and 500 HTTP errors
  - Graceful handling of runtime application errors

### Authentication

Terracotta features a custom-rolled authetication system, to demonstrate how to
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
from.

The authentication system is set up to make it easy to configure routes as
either public or protected, and is fully-implemented including a login page,
logout action, and handling of every part of the authentication journey and the
possible situations.

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


## Setup

The steps to set up this project are simple and standard. You need a
reasonably-recent Rust environment, on a Linux machine. There are currently no
special requirements beyond what is needed to build a standard Rust project.

### Environment

There are some key points to note about the environment you choose:

  - Debian and Ubuntu are the Linux distros of choice, although other distros
    should also work just fine, as there are no special requirements.
  - Running natively on Windows is not targeted or tested, and there are no
    plans to support it, so although it may work, it also may not. Running on
    WSL does work fine, and is the recommended way to run on Windows.
  - Running natively on MacOS is untested, although there is no known technical
    reason why it would not work.

Typically, you will set up Rust using [`rustup`](https://rustup.rs/), which is
the recommended way to install Rust. The `stable` toolchain is targeted, as the
focus is on stability and correctness, rather than bleeding-edge features.

Once you have Rust installed, you can build the project using `cargo build`.
This will download and compile all dependencies, and build the project. You can
then run the project using `cargo run`.

### Configuration

Terracotta is configured using a TOML file. The default configuration file is
`Config.toml`, which should be placed in the same directory as the binary. The
configuration settings (and file) are optional, and if not provided, Terracotta
will use default values for all configuration options.

It is also possible to pass configuration parameters from the command line, as
environment variables. The environment variables take precedence over the
configuration file options.

### Running

Terracotta can be run using the `cargo run` command, or by running the compiled
binary directly. The server will listen on port 8000 by default, and will serve
content from the `static` directory, plus any request handlers that you define.
The `static` directory contains the static files to be served.

### Testing

You can run the test suite using `cargo test`. This will run all unit and
integration tests.

**Note that, at present, there are no tests written specifically for this
project, as it is mostly a combination of other crates from the Rust ecosystem.
Tests might be added when the project is more mature and sensible things to test
have been clearly identified.**

### Documentation

This is the first release, so there is not much in the way of documentation just
yet. A few things may change when Axum 0.7 comes out, so documentation will be
written once Terracotta has been updated to be compatible.

You can build the developer documentation using `cargo doc`. This will generate
HTML files and place them into `target/doc`. You can then open the documentation
in your browser by opening `target/doc/terracotta/index.html`.

Building the documentation for local development use will also provide you with
links to the source code.


## Usage

The repository is designed so that it can be forked, and then customised and
extended. You will naturally rename the project and tailor it to your needs, and
as you implement your own features it will get harder and harder to merge in any
upstream changes. It is therefore likely best to consider this a starting point
only, and an upgrade reference, rather than an on-going contributing source.


### Structure

The code in this repository follows a simple and straightforward layout, which
is intended to be easy to understand and extend. You should absolutely modify
the file structure to suit the shape of your own web application.

The basic folder structure is:

  - `html`: This is where all the HTML templates reside, to be processed by
    Tera.
  - `src`: This is where all the Rust code lives.
  - `static`: This is where any static files should go, which are public and do
    not require authentication.

The layout of each folder should be fairly self-explanatory, but it is worth
mentioning that the `src` folder represents the simplest sensible minimum. In a
proper application it is likely that the handlers should be split out into more
files, and there would also be various other supporting files too. Rather than
dictate a layout, it is left as an exercise for the reader to implement their
preferred approach.

## Deployment

You can build the project in release mode by using `cargo build --release`.
Everything required for deployment will be contained in the single binary file
produced. It is recommended to run [`upx`](https://upx.github.io/) on the
executable before deployment, to reduce the file size.

The resulting binary file can then be copied to the deployment environment, and
run directly. This will often be in a Docker or Kubernetes container, but that
is outside the scope of this document.

A typical build script might look like this:

```sh
cargo build --release
upx --best target/release/terracotta
scp target/release/terracotta you@yourserver:/path/to/deployment/directory
```

## Attributions

This project uses the [Bulma CSS framework](https://bulma.io/), which is
[published](https://github.com/jgthms/bulma/blob/master/LICENSE) under the
[MIT license](http://opensource.org/licenses/MIT) and free to use without
restriction.

The [Font Awesome](https://fontawesome.com/) icons are [published](https://fontawesome.com/license/free)
under the [CC-BY (Creative Commons Attribution) license](https://creativecommons.org/licenses/by/4.0/),
and the webfonts under the [SIL OFL (Open Font License)](https://scripts.sil.org/OFL).
They are freely usable, along with the CSS code used to display them, which is
released under the [MIT license](http://opensource.org/licenses/MIT).


