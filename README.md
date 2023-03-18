<p align="center">
    <img align="center" width="280" src="zine.svg">
</p>

# zine

[![Crates.io](https://img.shields.io/crates/v/zine.svg)](https://crates.io/crates/zine)
![Crates.io](https://img.shields.io/crates/d/zine)
[![license-apache](https://img.shields.io/badge/license-Apache-yellow.svg)](./LICENSE)
[![dependency status](https://deps.rs/crate/zine/latest/status.svg)](https://deps.rs/crate/zine)

Zine - a simple and opinionated tool to build your own magazine.

https://zineland.github.io

- Mobile-first.
- Intuitive and elegant magazine design.
- Best reading experiences.
- Theme customizable, extend friendly.
- RSS Feed supported.
- Open Graph Protocol supported.
- Article topic supported.
- I18n and l10n supported.
- Build into a static website, hosting anywhere.

## Installation

`cargo install zine`

or `brew install zineland/tap/zine`

or `brew tap zineland/tap`, then `brew install zine`

## Get Started

Run `zine new your-zine-site`, you'll get following directory:

```
$ tree your-zine-site
your-zine-site
├── content             # The content directory your issues located
│   └── issue-1         # The first issue directory
│       ├── 1-first.md  # The first markdown article in this issue
│       └── zine.toml   # The issue Zine config file
└── zine.toml           # The root Zine config file of this project

2 directories, 3 files
```

Run `zine serve` to preview your zine site on your local computer:

```
$ cd your-zine-site

$ zine serve

███████╗██╗███╗   ██╗███████╗
╚══███╔╝██║████╗  ██║██╔════╝
  ███╔╝ ██║██╔██╗ ██║█████╗
 ███╔╝  ██║██║╚██╗██║██╔══╝
███████╗██║██║ ╚████║███████╗
╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝

listening on http://127.0.0.1:3000
```

Run `zine build` to build your zine site into a static website:

```
$ cd your-zine-site

$ zine build
Build success! The build directory is `build`.
```

## Some cool magazines powered by Zine

- [https://zineland.github.io](https://zineland.github.io) The zine documentation is built by zine itself.
- [https://rustmagazine.org](https://rustmagazine.org) The Rust Magazine.
- [https://2d2d.io](https://2d2d.io)
- [https://o11y.cn](https://o11y.cn)
- [https://thewhitepaper.github.io](https://thewhitepaper.github.io)

## Docmentations

- [Getting started](https://zineland.github.io/getting-started)
- [Customization](https://zineland.github.io/customization)
- [Code blocks](https://zineland.github.io/code-blocks)
- [Advanced](https://zineland.github.io/advanced)

## TODO

- [x] Support RSS Feed
- [x] Support render OGP meta
- [x] Support l10n
- [x] Support sitemap.xml
- [x] Support code syntax highlight
- [x] Support table of content
- [x] Support i18n
- [x] `zine serve` support live reload
- [x] Support article topic

## License

This project is licensed under the [Apache-2.0 license](./LICENSE).
