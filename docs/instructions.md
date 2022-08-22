# Local development

Clone this repository onto your local machine. You can run `cargo build` to build the
executable and `cargo test` to run the unit tests. When developing, you can use
`cargo run [file]` to run the program to edit the given file (if specified).


# Controls

## Default Mode
| Keybinding   | Function             |
|--------------|----------------------|
| `Ctrl+Q `    | Quit                 |
| `Ctrl+S`     | Save                 |
| `Ctrl+L`     | Search mode          |
| `Up/Down`    | Line up/down         |
| `Left/Right` | Character left/right |
| `Alt+Q/W`    | Word left/right      |
| `Alt+B/F`    | Line left/right      |
| `Alt+T/G`    | Page up/down         |
| `Home/End`   | Document up/down     |

## Search Mode
| Keybinding   | Function                                       |
|--------------|------------------------------------------------|
| `Up/Left`    | Search backwards                               |
| `Down/Right` | Search forwards                                |
| `Ctrl+F`     | Add selection and search forwards              |
| `Ctrl+B`     | Add selection and search backwards             |
| `Ctrl+D`     | Delete selections                              |
| `Ctrl+R`     | Replace selections                             |
| `Enter`      | Exit search mode (stay at current position)    |
| `Escape`     | Exit search mode (return to previous position) |
