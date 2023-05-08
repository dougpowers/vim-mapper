## Current Tasks
### Major
- [x] Implement one input manager per tab
- [x] Implement sheets in tabs
    - [x] Implement dedicated colors for active, inactive tabs and active tab indicator
    - [x] Implement tab clicking
- [x] Implement prompt mode for VMDialog
    - [x] Implement initial tab naming via VMDialog
    - [x] Implement tab renaming via VMDialog
    - [x] Implement tab deletion via VMDialog
- [x] Implement saving multiple sheets per file
    - [x] Implement importing VMSaveVersion into single-tab VMSaveVersion5
- [x] Implement custom text entry with vim-like bindings
    - [x] Implement UnicodeSegmentation-based iteration
- [x] Move VMTextInput from VimMapper member to VMInputManager member
- [x] Change regex keybinds to straight string rebinds
- [x] Add string keybind functionality to KeybindMode::Edit
- [x] Add functionality to split tree to external
- [x] Add functionality to split tree to new tab
- [x] Implement cross-tab cut/paste registers 
- [x] Add VMGraphClip rotation and translation logic
- [x] Add node insertion between any active and target nodes
- [x] Implement KeybindType::String for KeybindMode::Sheet
- [x] Implement text cursor placement on label click
- [x] Remove and replace mouse logic
- [x] Implement context menu for node operations via mouse
    - [x] Implement mouse node dragging
- [x] Implement sheet context menu
    - [x] add external node at mouse click point
    - [x] paste external at mouse click point
    - [x] paste into new tab
- [x] Implement tab context menu
- [x] Zoom from canvas center point rather than origin
- [x] Change zoom to vec of uniform scales
- [x] Implement array-based zoom levels
    - [x] refine zoom levels to make them smoother and more uniform
    - [x] fix extreme zoom levels scrolling the canvas absurdly offscreen
- [x] determine what permanently halts animation on HP Spectre x360
- [x] Stop node from resizing and repositioning during text entry
- [x] Implement node and tree yank
- [x] Fix tab context delete option always deleting the active tab
- [x] refactor vm_force_graph_rs deletion tree building logic
- [x] Add search list rendering
- [ ] Rewrite vim-mapper.vmd
- [ ] Rewrite README.md with modern template

### Minor
- [x] Fix partial matches being falsely reported
- [x] Fix delete to end of word leaving cursor in the wrong position when word is at end of string
- [x] Add functionality to save active_idx in each tab
- [x] Ensure any node deletion activates a convenient neighbor node instead of the root
- [x] Reposition mode indicators, prompts, and input
- [x] Change single click on node to target if it's in the target list
- [x] Fix brand new nodes sometimes not registering clicks
- [x] Add `A` and `I` keybinds for Sheet mode to skip to Insert mode at beginning and end of text
- [x] Add `C-J` and `C-K` to Move mode
- [x] Handle `Alt-F12` globally and dispatch to all tabs
- [x] Fix default zoom index
- [x] Fix search term truncation
- [x] Add double click text index selection for all nodes

## Deferred Tasks
### Major
- [ ] Implement visual mode
- [ ] Implement text registers
    - [ ] Implement system copy/paste
- [ ] Add merge tab feature, placing root node tree into register before deleting tab
- [ ] Implement global and local counts
- [ ] Implement :commands
- [ ] Implement new KeybindMode::RegisterSelect
- [ ] Show context menu for active node on Key::ContextMenu press
- [ ] Implement GraphClip registers
- [ ] Implement external file tab imports

### Minor
- [ ] Rewrite Delete/Change OuterN to delete empty pairs
