[![license](https://img.shields.io/crates/l/druid)](https://github.com/dougpowers/vim-mapper/blob/main/LICENCE)
## Vim-Mapper - A simple vim-inspired brainstorming tool.

Vim-Mapper is a graph editor with vim-like keybindings. It uses a force-directed algorithm to dynamically position nodes. Its goal is to allow a user create a node-tree for any purpose at the speed of thought without moving their hands from the home row of the keyboard.

## Compiling

Install [rustup](https://rustup.rs)

Ensure that rustc v1.66.0 is installed

```
rustup toolchain install 1.66.0
```

For WSL2 cross-compilation or Linux compilation on Debian, install librust-gtk-dev

```
sudo apt install librust-gtk-dev
```

Other packages like build-essentials may be necessary if a Linux build environment doesn't exist. For other distros, please search for the relevant rust GTK dev libraries.


Clone the repository

```
git clone https://github.com/dougpowers/vim-mapper vim-mapper 
```

Compile Vim-Mapper

```
cd vim-mapper
cargo build --release
```


## How to use Vim-Mapper

Vim-Mapper presents a simple interface. All new sheets will start with a single node called "Root". Subsequent nodes will connect back to this root node. 

Create a child to the active node by pressing 'O'. To simultaneously create, activate, and edit a child, press 'o'.

The active node is outlined in blue. The current targeted child node is outlined in light red. Press the 'n' key to cycle target clockwise or 'N' to cycle counter-clockwise. Press 'Enter' to make the targeted node active.

To edit the active node, press 'a', 'i', or 'c' to append after, insert before, or select the full node text, respectively.

To delete a node and its ancestors, press 'd'. A confirmation dialog will be displayed if more than one node is to be removed.

When executing Vim-Mapper from the terminal, the user can open an existing sheet by specifying a valid .vmd file as the first argument. 

## Advanced features

### Snipping
A non-root node in a linear chain (one that has only two neighbors) can be removed and its neighbors joined by pressing 'x'.

### External Nodes
New root nodes ("externals") can be created with 'Ctrl-Shift-o'. These appear on top of the root node in move mode. Move them using 'hjkl' or 'HJKL' and press 'Enter' to place them. These nodes constitute a new "component" that is unconnected and therefore non attracted to the default component (though they still push on nearby nodes from all components). New root nodes will be assigned a numerical mark from 1 to 9, corresponding to their respective component indices. These indices will shift to remain contiguous if an external is removed. Externals past index 9 are supported but will not be marked and can only be selected via [searching](#Searching).

### Marking
Vim-Mapper allows the user to "mark" a non-root node with any non-numeric printable character. Press 'm' to enter marking mode then press any printable character to mark that node. Marking any non-root node with ' ' (Space) will clear its mark. Press ''' (Apostrophe) to enter mark jump mode then press any non-numeric printable character to activate the node marked with that character.

Root nodes will have an unchangeable numeric mark corresponding to the index of the component. This mark may change if external nodes are removed.

Note the red "m" or "'" indicator in the bottom-left of the screen denoting that the user is now in marking or mark jump mode. These modes expire after 3 seconds of no input.

### Searching
Nodes can be navigated to via a case-insensitive text search. Press '/' and enter a string to begin filtering all nodes. Nodes that do not match will be grayed out as the string is entered. Press 'Enter' to begin search result navigation. Press 'n' or 'N' to cycle through matched nodes and press 'Enter' to select the desired match. If only one node matches the search string, pressing 'Enter' will skip result navigation and select it directly.

### Node Movement
Nodes can be moved by pressing '`' (Backtick). This anchors the node and enables move mode. Press 'hjkl' or 'HJKL' to move the node around the canvas. Pressing '@' will unanchor the node and cancel move mode. Pressing 'Enter' will confirm the new position for the node. Subsequently unanchoring the node will cause it once again to reposition relative to its connected and unconnected neighbor nodes.

### Mass
Vim-Mapper nodes have a default "mass" which affects how much other nodes are pushed away from it. Press the '+' or '-' keys to increment or decrement this mass for the active node. Press the '=' key to return the node to its default mass. A "+" or "-" badge will appear on the node if its mass is above or below the default.

### Anchoring
The root node of a Vim-Mapper sheet will be anchored by default. This means that it will not move in relation to any other node. All components must have at least 1 anchored node and Vim-Mapper will not allow a deletion if any of the removed nodes are the sole anchored node in that component. New child nodes are, by default, unanchored. The user can toggle the anchoring state of any node by pressing the '@' key. A "@" badge will appear on the node to indicate that it is anchored.

### Color Scheme
Vim-Mapper supports dark mode. It will attempt to detect your OS mode on first start-up. If this fails, the user can press 'Alt+F10' to toggle between dark mode and light mode. This preference will be saved.

### Hiding the Main Menu
If the user feels comfortable with the keybindings provided, they can hide the "File" menu by pressing 'Alt-F11'. This may make dark mode more complete and provide a cleaner interface in a maximized screen. This preference is saved.

### Changing UI Colors
Vim-Mapper stores its configuration in JSON format at ~/AppData/Roaming/vim-mapper/vmconfig on Windows and ~/.config/vim-mapper/vmconfig on Linux. This file can be edited manually to change color values but this is only recommended for advanced users. New versions of Vim-Mapper may not persist these custom changes and malformed configurations may cause unindended behavior or crashes.

## Keybindings
| **Key Combination** | **Mode**     | **Description**                                                                                                         |
|---------------------|--------------|-------------------------------------------------------------------------------------------------------------------------|
| Ctrl-n              | Any          | Create new sheet, discarding the current sheet                                                                          |
| Ctrl-o              | Any          | Open an existing sheet, discarding the current sheet                                                                          |
| Ctrl-s              | Any          | Save sheet                                                                                                              |
| Ctrl-Shift-s        | Any          | Open a dialog to save sheet to specific file                                                                            |
| Enter               | Edit         | Submit node change                                                                                                      |
| Esc                 | Edit         | Cancel node change                                                                                                      |
| Enter               | Sheet        | Set targeted child node as active                                                                                       |
| n                   | Sheet        | Cycle clockwise target through child nodes                                                                              |
| N                   | Sheet        | Cycle counter-clockwise through child nodes                                                                             |
| i                   | Sheet        | Edit active node, placing the caret at the beginning of the text                                                        |
| a                   | Sheet        | Edit active node, placing the caret at the end of the text                                                              |
| c                   | Sheet        | Edit active node, selecting the full text                                                                               |
| o                   | Sheet        | Create new child node, set as active, and edit                                                                          |
| O                   | Sheet        | Create new child node                                                                                                   |
| Ctrl-Shift-o        | Sheet        | Create a new external root node and enter move mode                                                                     |
| d                   | Sheet        | Delete node and children radiating away from root (displays confirmation dialog if more than one node is to be deleted) |
| x                   | Sheet        | Remove a node with only two neighbors and join them together                                                            |
| gg                  | Sheet        | Center viewport on active node                                                                                          |
| G                   | Sheet        | Center viewport on root node                                                                                            |
| j / J               | Sheet        | Pan the viewport down by a little / a lot                                                                               |
| k / K               | Sheet        | Pan the viewport up by a little / a lot                                                                                 |
| h / H               | Sheet        | Pan the viewport left by a little / a lot                                                                               |
| l / L               | Sheet        | Pan the viewport right by a little / a lot                                                                              |
| Ctrl-j              | Sheet        | Zoom the viewport out                                                                                                   |
| Ctrl-k              | Sheet        | Zoom the viewport in                                                                                                    |
| /                   | Sheet        | Enter search entry mode                                                                                                 |
| Enter               | Search Entry | Enter search mode and begin result navigation                                                                           |
| Esc               | Search Entry | Cancel search entry navigation                                                                           |
| n                   | Search       | Cycle forward through search results                                                                                    |
| N                   | Search       | Cycle backward throug search results                                                                                    |
| Enter               | Search       | Activate selected search result                                                                                         |
| Esc                 | Search       | Cancel search result navigation                                                                                                           |
| `                   | Sheet        | Enter move mode for the active node and anchor it                                                                       |
| j / J               | Move         | Move the node down by a little / a lot                                                                                  |
| k / K               | Move         | Move the node up by a little / a lot                                                                                    |
| h / H               | Move         | Move the node left by a little / a lot                                                                                  |
| l / L               | Move         | Move the node right by a little / a lot                                                                                 |
| @                   | Move         | Cancel move and unanchor node                                                                                           |
| +                   | Sheet        | Increase node mass                                                                                                      |
| -                   | Sheet        | Decrease node mass                                                                                                      |
| =                   | Sheet        | Reset node mass                                                                                                         |
| @                   | Sheet        | Anchor the active node                                                                                                  |
| m<char\>            | Sheet        | Mark the active node with <char\>                                                                                       |
| m<Space\>           | Sheet        | Clear the mark on the active node                                                                                       |
| '<char\>            | Sheet        | Jump to the node marked with <char\>                                                                                    |
| Alt+F10             | Sheet, Start | Toggle between dark and light mode                                                                                      |
| Alt+F11             | Sheet, Start | Hide app menu                                                                                                           |

## Mouse Controls
Vim-Mapper is intended to be used via efficient keybinds but basic mouse controls are supported. Not all features are accessible through these mouse controls. Refer to [Keybindings](Keybindings) for how to use these features.

Nodes can be activated by single left click. They can be edited by double left click. New child nodes can be created by right-clicking on the desired parent. The viewport can be panned by dragging while holding left click. The viewport can also be panned vertically by scrolling and horizontally by holding 'Shift' while scrolling. The viewport can be zoomed by holding right click or 'Ctrl' while scrolling.

## Acknowledgements
Vim-Mapper uses a forked version of the [force-graph-rs](https://github.com/t-mw/force-graph-rs) crate by [@tobmansf](https://twitter.com/tobmansf) to position and manage nodes. The [vm_force_graph_rs](https://github.com/dougpowers/vim-mapper/tree/main/vm-force-graph-rs) crate is not currently planned to be published on crates.io.

## Contact
Doug Powers - dougpowers@gmail.com - [LinkedIn](https://www.linkedin.com/in/douglas-powers-537380104)

Project Link: [https://github.com/dougpowers/vim-mapper](https://github.com/dougpowers/vim-mapper)
