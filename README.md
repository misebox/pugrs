# pugrs

is a transpiler written in Rust that compiles pugjs-like text into HTML.
Template engine feature like pug.js is not implemented.

This is a hobby project to get used to Rust.

## How to build
```
$ git clone https://github.com/misebox/pugrs
$ cd pugrs
$ cargo build --release
```

## Usage
```
$ target/release/pugrs samples/basic.pug
```

#### source (samples/basic.pug)

```
doctype html
html
  head
    meta(charset="UTF-8")
    title ページタイトル
  body
    .wrapper
      #header
        .menu
          a(href="#" alt="link"): img
      #container
        ul.item-list
          li.item text1
          li.item text2
          li.item text3
```

### Result
```
<html>
  <head>
    <meta charset="UTF-8">
    <title>
      ページタイトル
    </title>
  </head>
  <body>
    <div class="wrapper">
      <div id="container" id="header">
        <div class="menu">
          <a href="#" alt="link">
            <img>
          </a>
        </div>
        <ul class="item-list">
          <li class="item">
            text1
          </li>
          <li class="item">
            text2
          </li>
          <li class="item">
            text3
          </li>
        </ul>
      </div>
    </div>
  </body>
</html>
```
