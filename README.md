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

## Get Started

Run `zine new your-zine-site`, you'll get following directory:

```
$ tree your-zine-site
your-zine-site
├── content             # The content directory your seasons located
│   └── season-1        # The first season directory
│       ├── 1-first.md  # The first markdown article in this season
│       └── zine.toml   # The season Zine config file
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

This root `zine.toml` file describes your site meta and all your season's info.

```toml
[site]
url = "https://your-domain.com"
name = "Your Zine Site Name"
description = ""
logo = "path/to/favicon"
# the menu tabs
menu = [
    { name = "About", url = "/about" },
    { name = "Blog", url = "/blog" },
]

# You can customize some theme elements in this section.
# All of those elements are optional.
[theme]
# the primary color
primary-color = "#abcdef"
secondary-color = "#fff"
# the main text color
main-color = "#000"
# the link color in article content
link-color = "#e07312"
# the background image
background-image = "/static/background.png"
# you can customize your footer here
footer-template = "templates/footer.html"

# Season 1
[[season]]
# the slug of this season: https://your-domain.com/s1
slug = "s1"
# the number of this season
number = 1
# season title
title = "Season 1"
# the directory path to parse this season, you should put
# your markdown files in this directory
path = "content/season-1"
# the introduction of this season. optional.
intro = "content/season-1/intro.md"

# Season 2
[[season]]
slug = "s2"
number = 2
title = "Season 2"
path = "content/season-2"
```

### Season `zine.toml`

The season `zine.toml` file list all your articles of this season.

```toml
[[article]]
# the slug of this article. E.g: https://your-domain.com/s1/1
slug = "1"
# the markdown file path of this article
file = "1-first.md"
# the title of this article
title = "First article"
# the optional author of this article
author = ""
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

### Comment

You can add an arbitrary number of comments for an article. Simply put the **end matter** below the article content.

> **end matter** is a terminology similar to **front matter** in other Static Site Generators. Just like the **front matter** shown ahead of the markdown content, the **end matter** is shown below.

```markdown
You cool article content.

+++
[[comment]]
author = "Bob"
bio = "A developer"
content = "The cool comment"

[[comment]]
author = "Alice"
bio = ""
content = "Another cool comment"
+++
```

### Code blocks

Zine provides some advanced code blocks to help you write articles.

#### URL preview

~~~
```urlpreview
https://github.com/zineland/zine
```
~~~

## Some cool magazines powered by Zine

- [https://2d2d.io](https://2d2d.io)

## TODO

- [x] Support RSS Feed

- [x] Support render OGP meta

- [x] Support l10n

- [ ] Support table of content

- [ ] Support code syntax highlight

- [ ] Support i18n

- [ ] Generate word cloud for season

## License

This project is licensed under the [Apache-2.0 license](./LICENSE).