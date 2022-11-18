<p align="center">
    <img align="center" width="280" src="zine.svg">
</p>

# zine

[![Crates.io](https://img.shields.io/crates/v/zine.svg)](https://crates.io/crates/zine)
![Crates.io](https://img.shields.io/crates/d/zine)
[![license-apache](https://img.shields.io/badge/license-Apache-yellow.svg)](./LICENSE)

Zine - a simple and opinionated tool to build your own magazine.

- Mobile-first.
- Intuitive and elegant magazine design.
- Best reading experiences.
- Theme customizable, extend friendly.
- RSS Feed supported.
- Open Graph Protocol supported.
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

## Dive into deep

A Zine project mainly consists of two kind `zine.toml` files and a bunch of markdown files.

### Root `zine.toml`

This root `zine.toml` file describes your site meta and all your issue's info.

```toml
[site]
url = "https://your-domain.com"
name = "Your Zine Site Name"
description = ""
logo = "/path/to/favicon"
# the OpenGraph social image path.
social_image = "/path/to/social_image"
# the locale to localize your Zine site. default to "en".
# Zine has builtin supported locales, please check the `locales` directory of this repo.
locale = "en"
# the menu tabs
menu = [
    { name = "About", url = "/about" },
    { name = "Blog", url = "/blog" },
]

# Declare authors of this magazine.
[authors]
# set editor to true will show the Editor badge on the author profile page
zine_team = { name = "Zine Team", editor = true, bio = "The Zine Team." }

# You can customize some theme elements in this section.
# All of those elements are optional.
[theme]
# the primary color
primary_color = "#abcdef"
secondary_color = "#fff"
# the main text color
main_color = "#000"
# the link color in article content
link_color = "#e07312"
# the background image
background_image = "/static/background.png"
# you can customize your head here, such as your css, js
head_template = "templates/head.html"
# you can customize your footer here
footer_template = "templates/footer.html"
# you can extend the article page, for example adding comment widget
article_extend_template = "templates/article-extend.html"

# You can customize some markdown styles in this section.
# All of those elements are optional.
[markdown]
# enable the code highlighting. default is true
highlight_code = true
# custom highligh theme
highlight_theme = "ayu-light"

# Issue 1
[[issue]]
# the slug of this issue: https://your-domain.com/s1.
# optional. Fallback to `path` if missing. 
slug = "s1"
# the number of this issue
number = 1
# issue title
title = "Issue 1"
# the directory path to parse this issue, you should put
# your markdown files in this directory
dir = "content/issue-1"
# the introduction of this issue. optional.
intro = "content/issue-1/intro.md"

# Issue 2
[[issue]]
number = 2
title = "Issue 2"
dir = "content/issue-2"
```

### Issue `zine.toml`

The issue `zine.toml` file list all your articles of this issue.

```toml
[[article]]
# the slug of this article. E.g: https://your-domain.com/s1/1
# optional. Fallback to `file` name if missing.
slug = "1"
# the markdown file path of this article
file = "1-first.md"
# the title of this article
title = "First article"
# the optional author id of this article.
# also support multiple co-authors:
# author = ["author1", "author2"]
author = "zine-team"
# the cover of this article
cover = ""
# the publish date of this article
pub_date = "2022-03-20"
# whether to publish this article or not
publish = true
# whether mark this article as a featured article. 
# the featured articles will be shown on the home page
featured = true

# Another article
[[article]]

```

## Advanced

### Author

Zine will generate a dedicated profile page for each author declared in the root `zine.toml` table.

```toml
[authors]
# https://your-domain.com/@alice
alice = { name = "Alice", bio = "An engineer." }
# https://your-domain.com/@bob, the `avatar` and `bio` are optional.
bob = { name = "Bob", avatar = "/cool/avatar.png" }
```

> The path of an author page consists of `@` and author id, for example above, the path are `@alice` and `@bob`.
>
> If the author of an article hasn't declared in `[authors]`, no author page will be generated for that author.

### Pages

Every markdown file located in `pages` will be rendered as a **Page**. Just intuitive like this:

```
$ tree pages
pages
├── about.md        # will be rendered as https://your-domain.com/about
├── blog            
│   └── first.md    # will be rendered as https://your-domain.com/blog/first
├── blog.md         # will be rendered as https://your-domain.com/blog
└── faq.md          # will be rendered as https://your-domain.com/faq

1 directory, 4 files
```

### Code blocks

Zine provides some advanced code blocks to help you write articles.

#### Author

The author code is designed to render the avatar-name link on the markdown page.

The syntax is very simple, just write like this \`@author_id\`.
If the `author_id` is declared in the `[authors]` table of the root `zine.toml`, 
it will render the UI as expected, otherwise it fallback into the raw code UI.

#### Callout

#### Article link

#### URL preview

~~~
```urlpreview
https://github.com/zineland/zine
```
~~~

## Some cool magazines powered by Zine

- [https://2d2d.io](https://2d2d.io)
- [https://o11y.cn](https://o11y.cn)
- [https://rustpr.github.io](https://rustpr.github.io)

## TODO

- [x] Support RSS Feed

- [x] Support render OGP meta

- [x] Support l10n

- [x] Support sitemap.xml

- [x] Support code syntax highlight

- [x] Support table of content

- [ ] Support i18n

## License

This project is licensed under the [Apache-2.0 license](./LICENSE).