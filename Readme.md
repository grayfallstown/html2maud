# html2maud

Library / CLI / GUI to convert html to [maud]() template code.

# Binary Usage

`cargo install html2maud-bin`

`html2maud --in my.html --out my.maud`

`cat my.html | html2maud`

```
Usage: html2maud [OPTIONS]

Options:
  -i, --in <input html file>
  -o, --out <output maud file>
  -s, --stdin
  -h, --help                    Print help information
```

# GUI

Yet to be implemented.

# Library Usage

```
use html2maud::*;

fn main() {
    let html = "<div><p>Paragraph</p></div>";
    let maud = html2maud(html);
    println("{}", maud);
}
```

# LICENCE

MIT or Apache 2
