# Developer documentation

This file contains information of use to developers wanting to use a Terracotta
project template in their own projects. The rules and information contained here
should serve as a good baseline for any project.

The main sections in this document are:

  - [Getting started](#getting-started)
  - [Setup](#setup)
  - [Codebase structure](#codebase-structure)
  - [API endpoint structure](#api-endpoint-structure)
  - [API endpoint commit checklist](#api-endpoint-commit-checklist)
  - [Coding standards](#coding-standards)
  - [Coverage](#coverage)


## Getting started

[Commits to forks]:          https://github.com/orgs/community/discussions/45474
[Create repo from template]: https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template
[Rustmark]:                  https://crates.io/crates/rustmark

The Terracotta project template repositories are set up as a template
repositories on GitHub, so that you can easily [click the "Use this template"
button to create a new repository based on it][Create repo from template]. You
can then clone your new repository and start working on it. This will give you a
starting point where you have all the project files, but none of the commit
history. This is the currently-recommended way to start a new project using
Terracotta (there are plans to also have a command-line tool at a later date).

You may instead decide that you want to fork the repository, or clone it and
then push it to a new repository. This is also fine, but you should be aware of
the following points:

  - You will have the full commit history, which may be useful, but it is
    specifically relevant to Terracotta, and so mention of e.g. release versions
    will be misleading as your project will most likely maintain its own,
    independent versioning scheme.
  - You will also have the various release version tags created on the relevant
    Terracotta project template repository, which will be misleading for the
    same reason, and likely conflict with your own tags.
  - There is no particular advantage to maintaining a Git tree association with
    the Terracotta project template as an upstream repository, and as the
    development you do on your application will lead to conflicts, you are best
    not be pulling in updates for it. You should instead apply any updates
    manually. This is the same as when using other web application scaffolds.
  - Forks on GitHub are treated as subsidiaries of the original repository, and
    not first-class repositories in their own right. For this reason, [commits
    made to forks don't count as contributions in user profiles][Commits to forks],
    which is not a desirable situation if you are starting a new project.

For these reasons, forking in the GitHub-recognised sense is not recommended,
and cloning and pushing to a new repository is only recommended if you are
comfortable with the caveats mentioned above.

To see an example of a project that has been created based on Terracotta, you
can look at the [Rustmark][] application. This shows how Terracotta can be used
as a starting point, and then extended to create a more complex application.
Note that in the case of Rustmark, the decision was made to actually fork
Terracotta at a stage before its initial release, as the commit history was
considered useful, and there were no release commits or tags to cause the
conflict issues mentioned above. However, after that point of inception, all
Terracotta updates have been applied manually, and it is not a "true" fork in
GitHub terms.


## Setup

[Rustup]: https://rustup.rs/

The steps to set up a Terracotta project are simple and standard. You need a
reasonably-recent Rust environment, on a Linux or Mac machine. There are
currently no special requirements beyond what is needed to build a standard Rust
project.

Note that these instructions are for building the application yourself, which
will usually be in context of having used [Terracotta as a template for a new
project](#getting-started). In this case these steps will apply for your project
too.

### Environment

There are some key points to note about the environment you choose:

  - Debian and Ubuntu are the Linux distros of choice, although other distros
    should also work just fine, as there are no special requirements.
  - Running natively on MacOS works fine.
  - Running natively on Windows is not targeted or tested, and there are no
    plans to support it, so although it may work, it also may not. Running on
    WSL does work fine, and is the recommended way to run on Windows.

Typically, you will set up Rust using [`rustup`][Rustup], which is the
recommended way to install Rust. The `stable` toolchain is targeted, as the
focus is on stability and correctness, rather than bleeding-edge features.

Once you have Rust installed, you can build the project using `cargo build`.
This will download and compile all dependencies, and build the project. You can
then run the project using `cargo run`.

### Configuration

Because configuration applies to the examples as well as to a new project, the
various options are covered in the [Configuration](docs/examples.md#configuration)
section of the examples documentation.

### Running

A Terracotta-based project can typically be run using the `cargo run` command,
or by running the compiled binary directly. The server will listen on port 8000
by default, and will serve content from the `static` directory, plus any request
handlers that you define. The `static` directory contains the static files to be
served.

### Testing

You can run the test suite using `cargo test`. This will run all unit and
integration tests.

**Note that, at present, there are very few tests written specifically for this
project, as it is mostly a combination of other crates from the Rust ecosystem.
Those tests that do exist are intended to be examples of approach, and are not
exhaustive. Additional tests might be added when the project is more mature, and
sensible things to test have been clearly identified.**

### Documentation

You can build the developer documentation using `cargo doc`. This will generate
HTML files and place them into `target/doc`. You can then open the documentation
in your browser by opening `target/doc/terracotta/index.html`.

Building the documentation for local development use will also provide you with
links to the source code.

### Deployment

You can find information on how to deploy a Terracotta-based project in the
[documentation for each project template](docs/examples.md). The README files
have details about Docker, for example.


## Codebase structure

The code in the examples and template repositories follows a simple and
straightforward layout, which is intended to be easy to understand and extend.
You should absolutely modify the file structure to suit the shape of your own
web application.

The basic folder structure is:

  - `docs`: This contains documentation.
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

Any images and other files that need to be protected by authentication should be
placed in the `content` directory. Public images should be placed in
`static/img`, and will be served from the `/img` URL path, similar to the CSS,
JS, and WebFont files.

All of the content and static material is included in the compiled binary,
making it very straightforward to deploy.


## API endpoint structure

Machine-consumable endpoints should be placed under a path of `/api`. Those that
have application functionality should be versioned, and placed under a path of
`/api/vX`, where `X` is the version number. For example, the first version of
the API is placed under `/api/v1`.


## API endpoint commit checklist

This section contains a checklist of things that are mandatory to do when adding
a new API endpoint.

**For each endpoint added**, the following need to also be added:

  - Rustdocs
  - Unit tests
  - OpenAPI documentation
  - Written documentation

**For each commit made**, the following need to pass without errors or warnings:

  - `cargo build`
  - `cargo clippy`
  - `cargo doc`
  - `cargo test`

**Before a new endpoint can be declared complete**:

  - A coverage report needs to be run and checked. See the [Coverage](#coverage)
    section for more details.


## Coding standards

[Coding standards]: https://github.com/danwilliams/standards-rs
[Nerd Font]:        https://www.nerdfonts.com/

The code in this repository follows some specific and opinionated [coding
standards][]. These mostly follow typical community conventions, but notable
differences are the use of tabs for indentation, the alignment of various terms
to aid readability, the use of comment headers to separate sections of code, and
the usage of [Nerd Font][] symbols in those headers to belay semantic meaning in
order to apply highlighting.

You may well dislike aspects of the coding style, which is fine — feel free to
change things, and make the code your own! Individuality is important.


## Coverage

[kcov]:          https://github.com/SimonKagstrom/kcov
[rust-coverage]: https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html#source-based-code-coverage
[Tarpaulin]:     https://crates.io/crates/cargo-tarpaulin

[Since Rust 1.60.0, coverage is supported natively][rust-coverage]. This means
that there is no need to use external tools such as [Tarpaulin][] or [kcov][] to
generate coverage reports, which is a huge improvement.

### Preparation

On a Debian or Ubuntu system, you will need to install the `grcov` package. You
will also need to install the `llvm-tools-preview` component for Rust, and
create a directory to store the coverage reports in.

```bash
sudo apt install grcov
rustup component add llvm-tools-preview
mkdir coverage
```

### Running

The following commands will run the tests and generate coverage reports. The
profile files are then deleted, as they are not needed. The commands will
generate reports in HTML and LCOV formats, the latter of which can be loaded
into various tools.

Note that the `--binary-path` is important, and needs to point to your build
directory. By default this will be under `./target`, but if you have changed
this, e.g. to store builds in a central location, then you will need to adjust
the path accordingly.

```bash
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o coverage/html
grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o coverage/tests.lcov
find . -name "*.profraw" -delete
```

### Viewing

The HTML report can be viewed by opening `coverage/html/index.html` in your
browser.


