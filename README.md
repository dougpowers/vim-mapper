[![license](https://img.shields.io/crates/l/druid)](https://github.com/dougpowers/vim-mapper/blob/main/LICENCE)
## Vim-Mapper - A simple vim-inspired brainstorming tool.

Vim-Mapper is an experimental graph editor with vim-like keybindings. It uses a force-directed graph algorithm to dynamically position nodes. Its goal is to allow a user to rapidly create a node-tree without moving their hands from the home row of the keyboard.

## Compiling

Install rustup

For WSL2 cross-compilation, install librust-gtk-dev

## How to use Vim-Mapper

Vim-Mapper presents a simple interface. All new sheets will start with a single node called "Root". Subsequent nodes will connect back to this root node. 

Create a child to the active node by pressing 'O'. To simultaneously create, activate, and edit a child, press 'o'.

The active node is outlined in blue. The current targeted child node is outlined in light red. Press the 'n' key to cycle target clockwise or 'N' to cycle counter-clockwise. Press 'Enter' to make the targeted node active.

To edit the active node, press 'c'.

To delete a node and its ancestors, press 'd'. A confirmation dialog will be displayed if more than one node is to be removed.

When executing Vim-Mapper from the terminal, the user can open an existing sheet by specifying a valid .vmd file as the first argument. 

## Advanced features

### External Nodes
New root nodes ("externals") can be created with 'Ctrl-Shift-o'. These appear on top of the root node in move mode. Move them using 'hjkl' or 'HJKL' and press 'Enter' to place them. These nodes constitute a new "component" that is unconnected and therefore non attracted to the default component. New root nodes will be assigned a numerical mark from 1 to 9, corresponding to their respective component indices. Externals past 9 in index are allowed but will not be marked and can only be selected via [searching](Searching).

### Marking
Vim-Mapper allows the user to "mark" each node with any non-numeric printable character. Press 'm' to enter marking mode then press any printable character to mark that node. Press ''' (apostrophe) to enter mark jump mode then press any non-numeric printable character to activate the node marked with that character.

Root nodes will have an unchangeable numeric mark corresponding to the index of the component.

Marking a non-root node with the space key will clear it.

Note the red "m" or "'" indicator in the bottom-left of the screen denoting that the user is now in marking or mark jump mode. These modes expire after 3 seconds of no input.

### Searching
Nodes can be navigated to via a case-insensitive text search. Press '/' and enter a string to begin filtering all nodes. Nodes that do not match will be grayed out as the string is entered. Press 'Enter' to begin search result navigation. Press 'n' or 'N' to cycle through matched nodes and press 'Enter' to select the desired match. If only one node matches the search string, pressing 'Enter' will skip result navigation and select it directly.

### Node Movement
Nodes can be moved by pressing '`' (backtick). This anchors the node and enables move mode. Press 'hjkl' or 'HJKL' to move the node around the canvas. Pressing '@' will unanchor the node and cancel move mode. Pressing 'Enter' will confirm the new position for the node. Subsequently unanchoring the node will cause it once again to reposition relative to its connected and unconnected neighbor nodes.

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
| Key Combination | Context                         | Description                                   |
|-----------------|---------------------------------|-----------------------------------------------|
| Ctrl-n          | Any                             | Create new sheet, discarding the current sheet |
| Ctrl-s          | Any                             | Save sheet                                    |
| Ctrl-Shift-s    | Any                             | Save sheet to a file                          |
| Enter           | Editor focused                  | Submit node change                            |
| Esc             | Editor focused                  | Cancel node change                            |
| n               | Sheet focused, no node active   | Select root node                              |
| Enter           | Sheet focused, node active      | Set targeted child node as active            |
| n               | Sheet focused, node active      | Cycle target through child nodes              |
| c               | Sheet focused, node active      | Edit active node                              |
| o               | Sheet focused, node active      | Create new leaf node, set as active, and edit |
| d               | Sheet focused, left node active | Delete leaf node                              |
| G               | Sheet focused                   | Center viewport on root node                  |
| j / J           | Sheet focused                   | Pan the viewport down by a little / a lot     |
| k / K           | Sheet focused                   | Pan the viewport up by a little / a lot       |
| h / H           | Sheet focused                   | Pan the viewport left by a little / a lot     |
| l / L           | Sheet focused                   | Pan the viewport right by a little / a lot    |
| Ctrl-j          | Sheet focused                   | Zoom the viewport out                         |
| Ctrl-k          | Sheet focused                   | Zoom the viewport in                          |
| +               | Sheet focused, node active      | Increase node mass                            |
| -               | Sheet focused, node active      | Decrease node mass                            |
| =               | Sheet focused, node active      | Reset node mass                               |
| @       | Sheet focused, node active | Anchor the active node              |
| m<char\> | Sheet focused, node active | Mark the active node with <char\>    |
| m<Space\> | Sheet focused, node active | Clear the mark on the active node    |
| '<char\> | Sheet focused              | Jump to the node marked with <char\> |
| Alt+F10         | App focused                     | Toggle between dark and light mode            |

## Mouse Controls
Vim-Mapper is intended to be used via efficient keybinds but basic mouse controls are supported. Not all features are accessible through these mouse controls. Refer to [link](Keybindings) for how to use these features.

Nodes can be activated by single left click. They can be edited by double left click. New child nodes can be created by right-clicking on the desired parent. The viewport can be panned by dragging while holding left click. The viewport can also be panned vertically by scrolling and horizontally by holding 'Shift' while scrolling. The viewport can be zoomed by holding right click or 'Ctrl' while scrolling.

## Acknowledgements
Vim-Mapper uses a forked version of the [force-graph-rs](https://github.com/t-mw/force-graph-rs) crate by [@tobmansf](twitter.com/tobmansf) to position and manage nodes. The [vm_force_graph_rs](https://github.com/dougpowers/vim-mapper/tree/main/vm-force-graph-rs) crate is not currently planned to be published on crates.io.

## Contact
Doug Powers - dougpowers@gmail.com - [LinkedIn](https://www.linkedin.com/in/douglas-powers-537380104)

Project Link: [https://github.com/dougpowers/vim-mapper](https://github.com/dougpowers/vim-mapper)
