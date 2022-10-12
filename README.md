[![license](https://img.shields.io/crates/l/druid)](https://github.com/dougpowers/vim-mapper/blob/main/LICENCE)
## Vim-Mapper - A simple vim-inspired brainstorming tool.

Vim-Mapper is an experimental graph editor with vim-like keybindings. It uses a force-directed graph algorithm to dynamically position nodes. Its goal is to allow a user to rapidly create a node-tree without moving their hands from the home row of the keyboard.

## How to use Vim-Mapper

Vim-Mapper presents a simple interface. All new sheets will start with a single, fixed node called "Root". All subsequent nodes will connect back to this root node. 

The active node is outlined in blue. The current targetted child node is outlined in light red. Press the 'n' key to cycle to the desired child node and then press 'Enter' to make that node active.

When executing Vim-Mapper from the terminal, the user can open a sheet by specifying a valid .vmd file as the first argument.

## Advanced features

### Marking
Vim-Mapper allows the user to "mark" each node with any printable character. Press 'm' to enter marking mode then press any printable character to mark that node. Press the apostrophe ("'") key to enter mark jump mode then press any printable character to activate the node marked with that character.

Marking a node with the space key will clear it.

Note the red 'm' or ''' indicator in the bottom-left of the screen denoting that the use is now in marking or mark jump mode. These modes exprire after 3 seconds.

### Mass
Vim-Mapper nodes have a default "mass" which affects how much other nodes are pushed away from it. Press the "+" or "-" keys to increment or decrement this mass for the active node. Press the "=" key to return the node to its default mass. A "+" or "-" badge will appear on the node if its mass is above or below the default.

### Anchors
The root node of a Vim-Mapper sheet will always be anchored. This means that it will not move in relation to any other node. New nodes are, by default, unanchored. The user can toggle the anchoring state of any node by pressing the "@" key. A "@" badge will appear on the node to indicate that it is anchored.

### Color Scheme
Vim-Mapper supports dark mode. It will attempt to detect your OS mode on first start-up. If this fails, the user can press Alt+F10 to toggle between dark mode and light mode. This preference will be saved.

## Keybindings
| Key Combination | Context                         | Description                                   |
|-----------------|---------------------------------|-----------------------------------------------|
| Ctrl-n          | Any                             | Create new sheet, discarding the current sheet |
| Ctrl-s          | Any                             | Save sheet                                    |
| Ctrl-Shift-s    | Any                             | Save sheet to a file                          |
| Enter           | Editor focused                  | Submit node change                            |
| Esc             | Editor focused                  | Cancel node change                            |
| n               | Sheet focused, no node active   | Select root node                              |
| Enter           | Sheet focused, node active      | Set targetted child node as active            |
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
Nodes can be activated by single left click. They can be edited by double left click. New child nodes can be created by right-clicking on the desired parent. The viewport panned by dragging while holding left click. The viewport can also be panned vertically by scrolling and horizontally by holding 'Shift' while scrolling. The viewport can be zoomed by holding right click or 'Ctrl' while scrolling.

## Contact
Doug Powers - dougpowers@gmail.com - [LinkedIn](https://www.linkedin.com/in/douglas-powers-537380104)

Project Link: [https://github.com/dougpowers/vim-mapper](https://github.com/dougpowers/vim-mapper)