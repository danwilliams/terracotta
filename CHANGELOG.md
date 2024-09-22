# Changelog

[Axum]:                https://crates.io/crates/axum
[Bulma]:               https://bulma.io/
[Docker]:              https://www.docker.com/
[Figment]:             https://crates.io/crates/figment
[Font Awesome]:        https://fontawesome.com/
[Hyper]:               https://crates.io/crates/hyper
[Keep a Changelog]:    https://keepachangelog.com/en/1.0.0/
[OpenAPI]:             https://www.openapis.org/
[RapiDoc]:             https://mrin9.github.io/RapiDoc/
[Redoc]:               https://redoc.ly/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
[Swagger]:             https://swagger.io/
[Tera]:                https://crates.io/crates/tera
[Tracing]:             https://crates.io/crates/tracing

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][], and this project adheres to
[Semantic Versioning][].


## 0.3.2 (22 September 2024)

### Added

  - Added loading of HTML templates from the local filesystem at runtime
      - New `utility::render()` function
      - New configuration settings
  - Added custom JS and CSS files

### Fixed

  - Added a missing OpenAPI type definition

### Changed

  - Amended page title headings to apply line via CSS instead of using `HR` tags
  - Tweaked fonts for main text plus `H1`/`H2`/`H3`
  - CSS tweaks


## 0.3.1 (21 September 2024)

### Added

  - Added `/api/version` endpoint to `health` module
  - Added MSRV (Minimum Supported Rust Version) in `Cargo.toml`, set to 1.81.0
  - Added standard linting configuration
  - Added missing documentation

### Changed

  - Upgraded from `axum-sessions` to `tower-sessions`
  - Upgraded to [Axum][] 0.7 and [Hyper][] 1.0
  - Updated [Font Awesome][] from 6.4.0 -> 6.6.0
  - Updated [Bulma][] from 0.9.4 -> 1.0.2
  - Adjusted logo/nav CSS for dark mode
  - Changed Titlecase configuration fields to lowercase
  - Improved utility functions
  - Updated crate dependencies


## 0.3.0 (28 October 2023)

### Added

  - Added `health` module
      - Added `/api/ping` endpoint
  - Added `stats` module
      - Added `/api/stats` endpoint with request count, response count, response
        times, open connections, memory usage, summary data per period, and
        breakdown per endpoint
      - Added `/api/stats/history` endpoint with type selector and from/limit
        constraints
      - Added `/api/stats/feed` websocket endpoint with type selector
      - Implemented using a central statistics queue and circular buffers for
        historical data
      - Per-second tick clock to keep statistics up-to-date
      - Configurable buffer sizes and summary periods
  - Added [OpenAPI][] functionality, including UIs for [Swagger][], [Rapidoc][],
    and [Redoc][]
  - Added developer documentation
  - Added API integration documentation

### Changed

  - Changed memory allocator to `jemalloc`
  - Improved error logging


## 0.2.0 (25 September 2023)

### Added

  - Added serving of protected assets from a `content` folder
  - Added loading of static assets from the local filesystem at runtime
    (configurable)
  - Implemented streaming of large static assets (configurable)
  - Added `Dockerfile` for building and running the application in a [Docker][]
    container
  - Added logo and example content

### Changed

  - Updated crate dependencies


## 0.1.2 (18 June 2023)

### Added

  - Added host option to config
  - Added Rustdoc source code documentation

### Changed

  - Improved README documentation


## 0.1.1 (15 June 2023)

### Changed

  - Updated crate dependencies
  - Improved README documentation


## 0.1.0 (14 June 2023)

### Added

  - Implemented [Axum][] web framework
  - Added [Figment][] to manage configuration
  - Added [Tokio Tracing][Tracing] for logging and event tracing
  - Added [Tera][] template engine
  - Added serving of static files
  - Added [Bulma][] CSS framework
  - Added [Font Awesome][]
  - Implemented authentication
  - Implemented error handling
  - Added README documentation


