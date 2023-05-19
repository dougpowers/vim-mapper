[](https://img.shields.io/github/license/dougpowers/vim-mapper)
[![](https://img.shields.io/badge/LinkedIn-Douglas%20Powers-blue)](https://www.linkedin.com/in/douglas-powers-537380104/)
<h2 align="center"><img src="https://raw.githubusercontent.com/dougpowers/vim-mapper/main/assets/web/logo.png" height="128"></h2>
<p align="center"><strong>Vim Inspired Visual Graph Creation</strong></p>

VimMapper is a visual graph editor with vim-like keybindings. It uses a force-directed algorithm to dynamically position nodes. Its goal is to allow a user create a tree of text nodes for any purpose at the speed of thought without manual positioning or moving their hands from the home row of the keyboard.

***

- ❓ Report missing features/bugs on [GitHub](https://github.com/dougpowers/vim-mapper/issues).
- 📃 Refer to [TODO.md](TODO.md) for finished and upcoming features.

## 💾 Installation
VimMapper can be installed by going to the [releases](https://github.com/dougpowers/vim-mapper/releases) page and downloading the latest binaries for your operating system. 

VimMapper currently supports 64-bit Windows natively and Linux via GTK, GDK, and Cairo. Most Linux distributions with GUIs include these libraries by default.

## ☑ Features
- Easy text node creation and navigation for rapid creation of related textual information
- Vim-style keybindings for text entry and editing
- Dynamic node placement using a dense force-directed algorithm
- Vim-style keybindings for all node operations 
- Quick navigation without resorting to mouse operations
- Copy/paste functionality to easily reorganize whole tree branches
- Search functionality to quickly find and select specific nodes even in dense trees
- Tabbed sheets to organize multiple unrelated trees in a single file and move nodes between them

## 👩‍💻 Building VimMapper for Yourself
VimMapper is Open Source Software licensed under the [Apache 2.0 Licence](https://www.apache.org/licenses/LICENSE-2.0.html). Feel free to modify, build, and redistribute VimMapper. The build environment is currently configured for Windows and Linux. To build VimMapper for MacOS, changes to the source code may need to be made. Full support for MacOS is planned for future releases.

First, install [rustup](https://rustup.rs).

Ensure that rustc v1.66.0 is installed:
```sh
$ rustup toolchain install 1.66.0
```

For WSL2 cross-compilation or Linux compilation on Debian, install librust-gtk-dev:
```sh
$ sudo apt install librust-gtk-dev
```

Other packages such as build-essentials may be necessary if a Linux build environment hasn't been configured. For non-Debian Linux distributions, please search the appropriate repositories for the relevant rust GTK dev libraries.

Clone the repository:
```sh
$ git clone https://github.com/dougpowers/vim-mapper vim-mapper 
```

Compile VimMapper:
```sh
$ cd vim-mapper
$ cargo build --release
```

Compiled binaries will be located in `./target/release/`.


## 📋 Basic VimMapper Usage
VimMapper presents a simple interface. All new sheets will start with a single tab and a single node named "Root". Subsequent nodes will connect back to this root node. 

### Navigation
VimMapper starts in Sheet mode. Keys pressed in this mode will navigate and manipulate whole nodes or node trees. The sheet can be panned by pressing `h`, `j`, `k`, or `l`. Holding `Shift` while pressing these will pan by a larger amount. `Ctrl-k` and `Ctrl-j` will zoom the sheet in or out.

The current active node is outlined in blue. The current targeted child node is outlined in light red. Press the `n` key to cycle target clockwise or `N` to cycle counter-clockwise. Press `Enter` to activate the targeted node.

Create a child to the active node by pressing `O`. To create and activate a child as well as switch to Insert mode, press `o`.

To cut a node and its ancestors, press `d`. A confirmation dialog will be displayed if more than one node is to be removed. All cut nodes are automatically copied ("yanked") and can be pasted elsewhere if desired using `p`.

### Editing Nodes
To edit the active node, press `c` to enter Edit mode. This is analogous to vim's Normal mode. Common vim bindings to navigate and edit text are available in this mode.

When in Edit mode, press `i` or `a` to enter Insert mode. Press `I` or `A` to enter Insert mode and place the caret at the beginning or end of the text, respectively.

For a full list of keybindings, please see [Keybindings](#keybindings).

### Mouse Operations
Though VimMapper is intended to be used via the keyboard, mouse operations are available. Clicking and dragging an empty space will pan the sheet as will scrolling or holding `Shift` while scrolling. Holding `Control` and scrolling will zoom the sheet in or out. Right clicking an empty space will open a context menu for creating or pasting external nodes. 

On most Windows laptop tackpads, vertical and horizontal scrolling will pan the sheet and pinching will zoom the sheet.

Nodes can be moved by dragging them around the screen. Nodes can be activated by left click and edited by double left click. The text caret can be moved by clicking on the text in Edit mode. Right clicking a node will open a context menu with various options.

### Terminal Usage
When invoked from a terminal, VimMapper can open existing sheets by specifying a valid .vmd file as the first argument. 

## ⚙ Advanced features
### Vim-Like Bindings and Modes
VimMapper, like Vim, is designed to be used by touch typists without movement of the fingers from the home row of the keyboard. As such, it uses modes to separate functionality and allow the same keys to be used for different functions. The current mode is shown in the bottom-right of the interface. 

VimMapper includes the following modes:

* Sheet - The default mode used for navigating between, yanking, and pasting nodes
* Move - Accessed using `` ` ``. Allows movement of a single node around the sheet.
* Edit - Accessed using `c`. Allows navigation through the text of the active node using standard Vim keybinds. Can be exited using `Esc` or `Enter`.
* Insert - Accessed using `i`, `I`, `a`, or `A`. Allows text insertion for nodes. The caret can be moved using the arrow keys. Text can be deleted using `Backspace` or `Delete`. Press `Esc` or `jj` to return to Edit mode or press `Enter` to return to Sheet mode.
* Search - Accessed using `/`. Entering a string in this mode will search all the non-active nodes in the current tab for a specific string (case-insensitively).
* Select - Accessed by pressing `Enter` after entering a search string in Search mode. Press `n` or `N` to cycle the target through search results. Press `Enter` again to active the targeted node.
* Mark - Accessed by pressing `m`. Pressing a printable character after entering this mode will mark any non-root node. Pressing `Space` will remove a mark from a node. If a mark is already in use, reusing that mark will remove it from the old node and place it on the new node.
* Jump - Accessed by press `'` (apostrophe). Pressing a non-numeric printable character after entering this mode will activate a node marked with that character. Pressing `0` will activate the default root node. Pressing `1`-`9` will jump to the root node of the corresponding tree index.

### Cutting, Yanking, and Pasting Node Trees
VimMapper supports copying ("yanking" in vim parlance) and pasting of nodes and node trees. A node and all its descendants can be yanked by pressing `yy`. A single node can be yanked by pressing `yi`.

Any node deletion operation is also a yank operation. This allows for quickly moving nodes or node trees around the sheet. Be aware of this behavior when performing yank and cut operations consecutively as yanked nodes in the clipboard may be inadvertently overwritten by deleted ones.

Press `p` to attach a yanked node or node tree to the active node. Press `P` to paste the node or node tree as a new external tree. Press `Ctrl-p` to paste the node or node tree into a new tab.

> ❔**FAQ** - What about Vim-style registers?  
> _Vim-style registers are planned for future releases. Currently, only a single register is available for nodes._

### Tabs
VimMapper supports tabbed sheets. Press `Ctrl-t` to create a new tab. Press `Ctrl-Tab` and `Ctrl-Shift-Tab` to move between tabs. Press `Ctrl-Shift-t` to create a new tab with a prompt to enter a tab name. Press `Ctrl-r` to rename the active tab. Press `Ctrl-w` to delete a tab. This operation cannot be undone and will remove any nodes in that tab. Use yanks to move any desired nodes to other tabs before deleting a tab.

### Node Snipping and Insertion
A non-root node in a linear chain (one that has only two neighbors) can be cut and its neighbors joined by pressing `x`. 

A node can be inserted between the active and target nodes by pressing `i`.

### Anchoring
The root node of a VimMapper sheet will be anchored by default and will not move in relation to any other node. All node trees must have at least 1 anchored node and VimMapper will not allow a cut operation if any of the removed nodes are the sole anchored node in that component. New child nodes are, by default, unanchored. The anchoring state of any node can be toggled by pressing the `@` key. A ⚓ badge will appear on the node to indicate that it is anchored.

### Node Movement
Nodes can be moved by pressing `` ` `` (backtick). This anchors the node and enables Move mode. Press `hjkl` or `HJKL` to move the node around the canvas. Pressing `@` will unanchor the node and exit Move mode. Pressing `Enter` will confirm the new position for the node and exit Move mode. Subsequently unanchoring the node will cause it once again to reposition itself relative to its connected and unconnected neighbor nodes.

### External Nodes
New root nodes ("externals") can be created with `Ctrl-Shift-o`. These appear on top of the root node in Mode mode. Move them using `hjkl` or `HJKL` and press `Enter` to place them. These nodes constitute a new "tree" that is unconnected and therefore not attracted to the default tree (though they will still repel each other). New root nodes will be assigned a numerical mark from 1 to 9, corresponding to their respective tree indices. These indices will shift to remain contiguous if an external is removed. External roots past index 9 are supported but will not be marked and can only be selected via the mouse or by [searching](#searching). If more than 10 trees are desired, consider creating a new tab.

### Marking
VimMapper allows the user to "mark" a non-root node with any non-numeric printable character. Press `m` to enter Mark mode then press any printable character to mark the active node. Marking any non-root node with `Space` will clear its mark. Press `'` (apostrophe) to enter Jump mode then press any non-numeric printable character to activate the node marked with that character.

Root nodes will have an unchangeable numeric mark corresponding to the index of the component. This mark may change if external nodes are removed.

Note the red `m` or `'` indicator in the bottom-left of the screen denoting that the user is now in Mark or Jump mode. Press `Esc` to exit Mark or Jump mode.

### Searching
Nodes can be navigated to via a case-insensitive text search. Press `/` to enter Search mode and type a string to begin filtering through all non-active nodes. Results will be displayed in a pane on the left of the interface. Nodes that do not match will be grayed out on the sheet as the string is entered. Press `Enter` to enter Select mode and begin search result navigation. Press `n` or `N` to cycle through matched nodes and press `Enter` to select the desired match. If only one node matches the search string, pressing `Enter` will skip Select mode activate it directly.

### Mass
VimMapper nodes have a default "mass" which affects how much other nodes are repelled by it. Press the `+` or `-` keys to increment or decrement this mass for the active node. Press the `=` key to return the node to its default mass. A `+` or `-` badge will appear on the node if its mass is above or below the default.

### Color Scheme
VimMapper supports dark mode. It will attempt to detect the OS theme on first start-up. If this fails, press `Alt+F10` to toggle between dark mode and light mode. This preference will be saved.

### Hiding the Main Menu
The "File" menu can be hidden by pressing `Alt-F11`. This may make dark mode more complete and provide a cleaner interface in a maximized screen. This preference is saved.

### Changing UI Colors
VimMapper stores its configuration in JSON format at `~/AppData/Roaming/vim-mapper/vmconfig` on Windows and `~/.config/vim-mapper/vmconfig` on Linux. This file can be edited manually to change color values but this is only recommended for advanced users. New versions of VimMapper may not persist these custom changes and malformed configurations may cause unintended behavior or crashes.

## ⌨ Keybindings
### Sheet and Node Operations
| **Key Combination** | **Mode**     | **Description**                                                                                                         |
|---------------------|--------------|-------------------------------------------------------------------------------------------------------------------------|
| Ctrl+n              | Any          | Create new sheet, discarding the current sheet                                                                          |
| Ctrl+o              | Any          | Open an existing sheet, discarding the current sheet                                                                    |
| Ctrl+s              | Any          | Save sheet                                                                                                              |
| Ctrl+Shift+s        | Any          | Open a dialog to save sheet to specific file                                                                            |
| Alt+F4              | Any          | Close VimMapper (displays a confirmation dialog if the sheet has unsaved changes)                                      |
| Ctrl+t              | Sheet        | Create a new tab                                                                                                        |
| Ctrl+Shift+t        | Sheet        | Create and name a new tab                                                                                               |
| Ctrl+Tab            | Sheet        | Select the next tab                                                                                                     |
| Ctrl+Shift+Tab      | Sheet        | Select the previous tab                                                                                                 |
| Ctrl+r              | Sheet        | Rename the active tab                                                                                                   |
| Ctrl+w              | Sheet        | Delete the active tab                                                                                                   |
| Enter               | Sheet        | Set targeted child node as active                                                                                       |
| n                   | Sheet        | Cycle clockwise target through child nodes                                                                              |
| N                   | Sheet        | Cycle counter-clockwise through child nodes                                                                             |
| j / J               | Sheet        | Pan the viewport down by a little / a lot                                                                               |
| k / K               | Sheet        | Pan the viewport up by a little / a lot                                                                                 |
| h / H               | Sheet        | Pan the viewport left by a little / a lot                                                                               |
| l / L               | Sheet        | Pan the viewport right by a little / a lot                                                                              |
| Ctrl+j              | Sheet        | Zoom the viewport out                                                                                                   |
| Ctrl+k              | Sheet        | Zoom the viewport in                                                                                                    |
| c                   | Sheet        | Enter Edit mode                                                                                                         |
| I                   | Sheet        | Enter Insert mode, placing the caret at the beginning of the text                                                       |
| A                   | Sheet        | Enter Insert mode, placing the caret at the end of the text                                                             |
| o                   | Sheet        | Create new child node, set as active, and enter Insert mode                                                             |
| O                   | Sheet        | Create new child node                                                                                                   |
| Ctrl+Shift+o        | Sheet        | Create a new external root node and enter Move mode                                                                     |
| i                   | Sheet        | Insert a new node between the active and target nodes                                                                   |
| d                   | Sheet        | Cut node and any children radiating away from root (displays confirmation dialog if more than one node is to be removed)|
| x                   | Sheet        | Cut a node with only two neighbors and join them together                                                               |
| yy                  | Sheet        | Yank a node tree                                                                                                        |
| yi                  | Sheet        | Yank a single node                                                                                                      |
| p                   | Sheet        | Attached a yanked node or node tree to the active node                                                                  |
| P                   | Sheet        | Paste a yanked node or node tree as a new external tree                                                                 |
| Ctrl+p              | Sheet        | Paste a yanked node or node tree as a new tab                                                                           |
| gg                  | Sheet        | Center viewport on the active node                                                                                      |
| G                   | Sheet        | Center viewport on the default root node                                                                                |
| /                   | Sheet        | Enter Search mode                                                                                                       |
| Enter               | Search       | Enter Select mode                                                                                                       |
| Esc                 | Search       | Cancel Search mode and return to Sheet mode                                                                             |
| n                   | Select       | Cycle forward through search results                                                                                    |
| N                   | Select       | Cycle backward through search results                                                                                   |
| Enter               | Select       | Activate selected search result                                                                                         |
| Esc                 | Select       | Cancel Select mode and return to Sheet mode                                                                             |
| `                   | Sheet        | Enter Move mode for the active node and anchor it                                                                       |
| j / J               | Move         | Move the node down by a little / a lot                                                                                  |
| k / K               | Move         | Move the node up by a little / a lot                                                                                    |
| h / H               | Move         | Move the node left by a little / a lot                                                                                  |
| l / L               | Move         | Move the node right by a little / a lot                                                                                 |
| @                   | Move         | Cancel Move mode and unanchor node                                                                                      |
| +                   | Sheet        | Increase node mass                                                                                                      |
| -                   | Sheet        | Decrease node mass                                                                                                      |
| =                   | Sheet        | Reset node mass                                                                                                         |
| @                   | Sheet        | Anchor the active node                                                                                                  |
| m<char\>            | Sheet        | Mark the active node with <char\>                                                                                       |
| m<Space\>           | Sheet        | Clear the mark on the active node                                                                                       |
| '<char\>            | Sheet        | Jump to the node marked with <char\>                                                                                    |
| Alt+F10             | Sheet, Start | Toggle between dark and light mode                                                                                      |
| Alt+F11             | Sheet, Start | Hide app menu                                                                                                           |

### Text Operations
| **Key Combination** | **Mode**       | **Description**                                                                                                         |
|---------------------|----------------|-------------------------------------------------------------------------------------------------------------------------|
| Enter / Esc         | Edit           | Exit Edit mode and return to Sheet mode                                                                                 |
| i                   | Edit           | Enter Insert mode                                                                                                       |
| a                   | Edit           | Enter Insert mode, advancing the carat one character to the right                                                       |
| I                   | Edit           | Enter Insert mode, placing the carat at the beginning of the text                                                       |
| A                   | Edit           | Enter Insert mode, placing the carat at the end of the text                                                             |
| s                   | Edit           | Remove the character under the carat and enter Insert mode                                                              |
| u                   | Edit           | Undoes the last change to the the text                                                                                  |
| Ctrl+r              | Edit           | If no changes have been made since the last undo, reverts the undo                                                      |
| l / Right Arrow     | Edit (Movement)| Advance the carat one character to the right                                                                            |
| h / Left Arrow      | Edit (Movement)| Advance the carat one character to the left                                                                             |
| w                   | Edit (Movement)| Advance the carat to the beginning of the next word                                                                     |
| e                   | Edit (Movement)| Advance the carat to the end of the next word                                                                           |
| ^                   | Edit (Movement)| Move the carat to the beginning of the text                                                                             |
| $                   | Edit (Movement)| Move the carat to the end of the text                                                                                   |
| t<char\>            | Edit (Movement)| Move the carat to the next occurrence of <char\>                                                                        |
| f<char\>            | Edit (Movement)| Move the carat past the next occurrence of <char\>                                                                      |
| r<char\>            | Edit           | Replace the character under the carat with <char\>                                                                      |
| D                   | Edit           | Delete text from the carat position to the end of the line                                                              |
| d<movement\>        | Edit           | Delete text from the current carat position to <movement\>                                                              |
| di<delimiter\>      | Edit           | Delete text under the carat between two [delimiters](#delimiters)                                                       |
| diw                 | Edit           | Delete the word underneath the carat and one leading or trailing whitespace character                                   |
| C                   | Edit           | Delete text from the carat position to the end of the line and enter Insert mode                                        |
| c<movement\>        | Edit           | Delete text from the current carat position to <movement\> and enter Insert mode                                        |
| ciw                 | Edit           | Delete the word underneath the carat and one leading or trailing whitespace character and enter Insert mode             |
| ci<delimiter\>      | Edit           | Delete text under the carat between two [delimiters](#delimiters) and enter Insert mode                                 |
| S                   | Edit           | Delete all text and enter Insert mode                                                                                   |
| Esc / jj            | Insert         | Exit Insert mode and enter Edit mode                                                                                    |
| Enter               | Insert         | Exit Insert mode and enter Sheet mode                                                                                   |
|                     | Edit           |                                                                                                                         |

> ❔**FAQ** - Where is my favorite vim command?  
> _VimMapper is not intended for extensive text entry and therefore only emulates a subset of vim functionality. Currently, support for more movements and well as numerical counts, sentence objects, visual mode, and other operations is planned for future releases. However, fully emulating vim or even vi is outside the scope of the VimMapper project._

> ❔**FAQ** - What about text registers?  
> _Text registers are planned for a future release._

### ‼ Compatibility Warnings and Missing Features
* As in vim, `dw` and `cw` are the same operation as `de` and `ce`, respectively.
* VimMapper currently uses the [unicode-segmentation](https://docs.rs/unicode-segmentation/1.10.1/unicode_segmentation/index.html) crate to determine words and whitespace. This behavior will differ in some ways to vim's words and WORDS.
* While it does support Unicode, VimMapper does not support non-LTR languages. Likewise, it does not support some IMEs.

### Delimiters
VimMapper recognizes `"`, `'`, `[` and `]`, `(` and `)`, `<` and `>`, and `{` and `}` as valid delimiters.

## 📣 Acknowledgements
VimMapper uses a forked version of the [force-graph-rs](https://github.com/t-mw/force-graph-rs) crate by [@tobmansf](https://twitter.com/tobmansf) to position and manage nodes. The [vm-force-graph-rs](https://github.com/dougpowers/vim-mapper/tree/main/vm-force-graph-rs) crate is not currently planned to be published on crates.io.

## 📧 Contact
Doug Powers - dougpowers@gmail.com - [LinkedIn](https://www.linkedin.com/in/douglas-powers-537380104)

Project Link: [https://github.com/dougpowers/vim-mapper](https://github.com/dougpowers/vim-mapper)
