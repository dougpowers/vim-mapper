use druid::{Widget, WidgetExt, piet::D2DTextLayout, Vec2, WidgetPod, widget::{Container, Controller, TextBox}, EventCtx, Event, Env, keyboard_types::Key, text::Selection};
use force_graph::DefaultNodeIdx;

use crate::constants::*;
pub struct VMNodeLayoutContainer {
    pub layout: Option<D2DTextLayout>,
    pub parent: Option<u16>, 
    #[allow(dead_code)]
    pub index: u16,
}

pub struct VMNode {
    pub label: String,
    pub edges: Vec<u16>,
    pub index: u16,
    pub fg_index: Option<DefaultNodeIdx>,
    pub pos: Vec2,
    pub container: VMNodeLayoutContainer,
    pub is_active: bool,
}

pub struct VMNodeEditor {
    #[allow(dead_code)]
    pub container: WidgetPod<String, Container<String>>,
    pub is_visible: bool,
    pub title_text: String,
}

pub struct VMNodeEditorController {
}

impl VMNodeEditorController {
    pub fn new() -> VMNodeEditorController {
        VMNodeEditorController {}
    }
}

impl Controller<String, TextBox<String>> for VMNodeEditorController {
    fn event(&mut self, child: &mut TextBox<String>, ctx: &mut EventCtx, event: &Event, data: &mut String, env: &Env) {
        match event {
            Event::KeyDown(event) if event.key == Key::Enter => {
                ctx.submit_notification(SUBMIT_CHANGES);
                ctx.resign_focus();
                ctx.set_handled();
            }
            Event::KeyUp(event) if event.key == Key::Enter => {
                ctx.set_handled();
            }
            Event::KeyDown(event) if event.key == Key::ArrowDown => {
                ctx.set_handled();
            }
            Event::Command(command) => {
                if command.is(TAKE_FOCUS) {
                    ctx.request_focus();
                    ctx.set_handled();
                    let mut selection = Selection::new(0,1000);
                    if let Some(text) = child.editor().layout().text() {
                        selection = selection.constrained(text);
                        println!("{:?}", selection);
                        selection.end = selection.max();
                    }
                    child.set_selection(selection);
                }
            }
            Event::MouseDown(_event) => {
                ctx.submit_notification(TAKEN_FOCUS);
                ctx.set_handled();
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
}

impl VMNodeEditor {
    pub fn new() -> VMNodeEditor {
        let padding = WidgetPod::<String, Container<String>>::new(
            Container::new(TextBox::<String>::multiline().controller(VMNodeEditorController::new())));
            
        let nodeeditor = VMNodeEditor {
            container: padding,
            is_visible: false,
            title_text: "".to_string(),
        };
        nodeeditor
    }
}

#[allow(dead_code)]
pub struct VMEdge {
    pub label: Option<String>,
    pub from: u16,
    pub to: u16,
    pub index: u16,
}

impl VMNodeLayoutContainer {
    pub fn new(_label: String, index: u16) -> VMNodeLayoutContainer {
        let new_layout = VMNodeLayoutContainer {
            layout: None,
            parent: None,
            index,
        };
        new_layout
    }

    #[allow(dead_code)]
    pub fn set_parent(&mut self, parent: u16) {
        self.parent = Some(parent)
    }
}