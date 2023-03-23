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
- [ ] Add functionality to split tree to external
- [ ] Add functionality to split tree to new tab
- [ ] Implement cross-tab cut/paste registers 
- [ ] Remove and replace mouse logic
- [ ] Implement context menu for node operations via mouse
    - [ ] Implement mouse node dragging
- [ ] Zoom from center point rather than origin

### Minor
- [x] Fix partial matches being falsely reported
- [x] Fix delete to end of word leaving cursor in the wrong position when word is at end of string
- [ ] Ensure any node deletion activates a convenient neighbor node instead of the root
- [ ] Reposition mode indicators, prompts, and input

## Deferred Tasks
### Major
- [ ] Implement global and local counts
- [ ] Implement visual mode
- [ ] Implement :commands

### Minor
- [ ] Rewrite Delete/Change OuterN to delete empty pairs