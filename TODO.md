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
- [ ] Implement KeybindType::String for KeybindMode::Sheet
- [ ] Implement node and tree yank
- [x] Remove and replace mouse logic
- [x] Implement context menu for node operations via mouse
    - [x] Implement mouse node dragging
- [ ] Implement sheet context menu
    - [ ] add external node at mouse click point
    - [ ] paste external at mouse click point
    - [ ] paste into new tab
- [ ] Implement tab context menu
- [ ] Zoom from canvas center point rather than origin

### Minor
- [x] Fix partial matches being falsely reported
- [x] Fix delete to end of word leaving cursor in the wrong position when word is at end of string
- [x] Add functionality to save active_idx in each tab
- [x] Ensure any node deletion activates a convenient neighbor node instead of the root
- [x] Reposition mode indicators, prompts, and input
- [ ] Fix brand new nodes sometimes not registering clicks

## Deferred Tasks
### Major
- [ ] Implement global and local counts
- [ ] Implement visual mode
- [ ] Implement :commands
- [ ] Implement new KeybindMode::RegisterSelect

### Minor
- [ ] Rewrite Delete/Change OuterN to delete empty pairs