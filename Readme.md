# html2maud

Library / CLI / GUI to convert html to [maud]() template code.


# Online version

[html2maud online](https://grayfallstown.github.io/html2maud/)

# Binary Usage

`cargo install html2maud-bin`

`html2maud-bin --in my.html --out my.maud`

`cat my.html | html2maud-bin --stdin`

```
Usage: html2maud [OPTIONS]

Options:
  -i, --in <input html file>
  -o, --out <output maud file>
  -s, --stdin
  -h, --help                    Print help information
html2maud-bin without parameters will start the GUI
```

# GUI

run `html2maud-bin` without parameters and a basic [tauri]("https://tauri.app/) + [pâro](https://github.com/grayfallstown/paro-rs) GUI will start, allowing for a faster workflow.

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
