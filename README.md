[![license](https://img.shields.io/crates/l/druid)](https://github.com/dougpowers/vim-mapper/blob/main/LICENCE)
## A simple vim-inspired brainstorming tool.

Vim-Mapper is an experimental graph editor with vim-like keybindings. Its goal is to allow a user to rapidly create a node-tree without moving their hands from the home row of the keyboard.

## How to use Vim-Mapper

Vim-Mapper presents a simple interface. All new sheets will start with a single, fixed node called "Root". All subsequent nodes will connect back to this root node. 

The active node is outlined in blue. The current targetted child node is outlined in light red. Press the 'n' key to cycle to the desired child node and then press 'Enter' to make that node active.

## Keybindings
| Key Combination | Context                         | Description                                   |
|-----------------|---------------------------------|-----------------------------------------------|
| Ctrl-n          | Any                             | Create new sheet (WARNING: WILL NOT SAVE CURRENT SHEET)                             |
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

## Mouse Controls
Nodes can be activated by single left click. They can be edited by double left click. New child nodes can be created by right-clicking on the desired parent. The viewport panned by dragging while holding left click. The viewport can also be panned vertically by scrolling and horizontally by holding 'Shift' while scrolling. The viewport can be zoomed by holding right click or 'Ctrl' while scrolling.

## Contact
Doug Powers - dougpowers@gmail.com - [LinkedIn](https://www.linkedin.com/in/douglas-powers-537380104)
Project Link: [https://github.com/dougpowers/LinkLocker](https://github.com/dougpowers/LinkLocker)