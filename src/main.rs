use druid::keyboard_types::Key;
use druid::kurbo::{Line, TranslateScale, Circle};
use druid::piet::{ Text, TextLayoutBuilder, TextLayout};
use force_graph::{ForceGraph, NodeData, EdgeData};
use druid::widget::{prelude::*};
use druid::{AppLauncher, Color, WindowDesc, FontFamily, Affine, Point, Vec2, Rect, WindowState, TimerToken, Command, Target};
use std::collections::HashMap;

mod vmnode;
use vmnode::{VMEdge, VMNode, VMNodeEditor, VMNodeLayoutContainer};

mod constants;
use crate::constants::*;


struct VimMapper {
    graph: ForceGraph<u16, u16>,
    animating: bool,
    nodes: HashMap<u16, VMNode>,
    edges: HashMap<u16, VMEdge>,
    node_idx_count: u16,
    edge_idx_count: u16,
    translate: TranslateScale,
    scale: TranslateScale,
    offset_x: f64,
    offset_y: f64,
    last_click_point: Option<Point>,
    last_collision_rects: Vec<Rect>,
    is_focused: bool,
    #[allow(dead_code)]
    node_editor: VMNodeEditor,
    is_dragging: bool,
    drag_point: Option<Point>,
    double_click_timer: Option<TimerToken>,
    double_click: bool,
    translate_at_drag: Option<(f64, f64)>,
}

impl<'a> VimMapper {
    pub fn new() -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(
            DEFAULT_SIMULATION_PARAMTERS
        );
        let mut root_node = VMNode {
            label: "Root".to_string(),
            edges: Vec::with_capacity(10),
            index: 0,
            fg_index: None,
            pos: Vec2::new(0.0, 0.0),
            container: VMNodeLayoutContainer::new("Root".to_string(), 0),
            is_active: true,
        };
        root_node.fg_index = Some(graph.add_node(NodeData { x: 0.0, y: 0.0, is_anchor: true, user_data: 0, ..Default::default() }));
        let mut mapper = VimMapper {
            graph: graph, 
            animating: true,
            nodes: HashMap::with_capacity(50),
            edges: HashMap::with_capacity(100),
            node_idx_count: 1,
            edge_idx_count: 1,
            translate: DEFAULT_TRANSLATE,
            scale: DEFAULT_SCALE,
            offset_x: DEFAULT_OFFSET_X,
            offset_y: DEFAULT_OFFSET_Y,
            last_click_point: None,
            last_collision_rects: Vec::new(),
            is_focused: true,
            node_editor: VMNodeEditor::new(),
            is_dragging: false,
            drag_point: None,
            translate_at_drag: None,
            double_click_timer: None,
            double_click: false,
        };
        mapper.nodes.insert(0, root_node);
        mapper
    }

    pub fn add_node(&mut self, from_idx: u16, node_label: String, edge_label: Option<String>) -> () {
        let new_idx = self.get_new_node_idx();
        let from_node = self.nodes.get_mut(&from_idx);

        let x_offset = (rand::random::<f64>()-0.5) * 10.0;
        let y_offset = (rand::random::<f64>()-0.5) * 10.0;
        match from_node {
            Some(from_node) => {
                let mut new_node = VMNode {
                    label: node_label.clone(),
                    edges: Vec::with_capacity(10),
                    index: new_idx,
                    fg_index: None,
                    pos: Vec2::new(from_node.pos.x + x_offset, from_node.pos.y + y_offset),
                    container: VMNodeLayoutContainer::new(node_label.clone(), new_idx),
                    is_active: false,
                };
                let new_edge: VMEdge;
                match edge_label {
                    Some(string) => {
                        new_edge = VMEdge {
                            label: Some(string),
                            from: from_node.index,
                            to: new_node.index,
                            index: self.edge_idx_count,
                        }
                    }
                    _ => {
                        new_edge = VMEdge {
                            label: None,
                            from: from_node.index,
                            to: new_node.index,
                            index: self.edge_idx_count
                        }
                    } 
                }
                new_node.fg_index = Some(self.graph.add_node(NodeData {
                    x: new_node.pos.x as f32,
                    y: new_node.pos.y as f32,
                    user_data: new_node.index,
                    ..Default::default()
                }));
                self.graph.add_edge(from_node.fg_index.unwrap(), new_node.fg_index.unwrap(), EdgeData { user_data: new_edge.index }); 
                new_node.edges.push(new_edge.index);
                from_node.edges.push(new_edge.index);
                self.nodes.insert(new_node.index, new_node);
                self.edges.insert(new_edge.index, new_edge);
            }
            _ => {
                panic!("Tried to add to a non-existent node")
            } 
        }
    }

    pub fn get_active_node(&mut self) -> Option<u16> {
        let active_node = self.nodes.iter().find(|item| {
            if item.1.is_active {
                true
            } else {
                false
            }
        });
        if let Some((idx, _node)) = active_node {
            Some(*idx)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn clear_active_node(&mut self) -> () {
        self.nodes.iter_mut().for_each(|item| {
            item.1.is_active = false;
        });
    }

    pub fn set_active_node(&mut self, idx: u16) {
        self.nodes.iter_mut().for_each(|item| {
            if item.1.index == idx {
                item.1.is_active = true;
            } else {
                item.1.is_active = false;
            }
        });
    }

    pub fn get_new_node_idx(&mut self) -> u16 {
        let idx = self.node_idx_count.clone();
        self.node_idx_count += 1;
        idx
    }

    pub fn update_node_coords(&mut self) -> () {
        self.graph.visit_nodes(|fg_node| {
            let node: Option<&mut VMNode> = self.nodes.get_mut(&fg_node.data.user_data);
            match node {
                Some(node) => {
                    node.pos = Vec2::new(fg_node.x() as f64, fg_node.y() as f64);
                }
                None => {
                    panic!("Attempted to update non-existent node coords from graph")
                }
            }
        });
    }

    pub fn does_point_collide(&mut self, point: Point) -> Option<u16> {
        self.last_collision_rects = Vec::new();
        self.last_click_point = Some(point);
        let mut add_to_index: Option<u16> = None;
        self.nodes.iter().for_each(|item| {
            let affine_scale = Affine::scale(self.scale.as_tuple().1);
            let affine_translate = Affine::translate(self.translate.as_tuple().0);
            let node = item.1;
            let size = node.container.layout.as_ref().unwrap().size();
            let mut rect = size.to_rect();
            let border = DEFAULT_BORDER_SIZE*self.scale.as_tuple().1;
            rect = rect.inflate(border*2.0,border*2.0);
            let pos_translate = Affine::translate(
                (affine_scale * (
                    node.pos - Vec2::new(size.width/2.0, size.height/2.0)
                ).to_point()).to_vec2()
            );
            rect = affine_scale.transform_rect_bbox(rect);
            rect = (affine_translate).transform_rect_bbox(rect);
            rect = (pos_translate).transform_rect_bbox(rect);
            self.last_collision_rects.push(rect);
            if rect.contains(point) {
                add_to_index = Some(node.index);
            }
        });
        add_to_index
    }

    pub fn set_dragging(&mut self, is_dragging: bool, drag_point: Option<Point>) -> () {
        if is_dragging {
            self.is_dragging = true;
            self.drag_point = drag_point;
            self.translate_at_drag = Some((self.offset_x, self.offset_y));
        } else {
            self.is_dragging = false;
            self.drag_point = None;
            self.translate_at_drag = None;
        }
    }
}

impl<'a> Widget<()> for VimMapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        if self.is_focused {
            ctx.request_focus();
        }
        self.node_editor.container.event(ctx, event, &mut self.node_editor.title_text, _env);
        match event {
            Event::AnimFrame(interval) => {
                ctx.request_paint();
                ctx.request_layout();
                for _ in 0..10 {
                    self.graph.update(((*interval as f32)) * 1e-9);
                }
                self.update_node_coords();
                if self.animating {
                    ctx.request_anim_frame();
                }
            }
            Event::MouseUp(event) => {
                self.set_dragging(false, None);
                if event.button.is_left() {
                    if let Some(_token) = self.double_click_timer {
                        self.double_click = true;
                    } else {
                        self.double_click_timer = Some(ctx.request_timer(DOUBLE_CLICK_THRESHOLD));
                    }
                }
            }
            Event::MouseDown(event) => {
                if self.does_point_collide(event.pos) == None {
                    if event.button.is_left() {
                        self.set_dragging(true, Some(event.pos));
                    }
                    self.is_focused = true;
                    // self.node_editor.is_visible = false;
                }
            }
            Event::MouseMove(event) => {
                if self.is_dragging {
                    if let Some(drag_point) = self.drag_point {
                        let delta = drag_point - event.pos;
                        self.offset_x = self.translate_at_drag.unwrap().0 - delta.x;
                        self.offset_y = self.translate_at_drag.unwrap().1 - delta.y;
                    }
                }
            }
            Event::Wheel(event) => {
                if event.mods.shift() {
                    self.offset_x -= event.wheel_delta.to_point().x;
                } else if event.mods.ctrl() {
                    if event.wheel_delta.to_point().y < 0.0 {
                        self.scale = self.scale.clone()*TranslateScale::scale(1.25);
                    } else {
                        self.scale = self.scale.clone()*TranslateScale::scale(0.75);
                    }
                } else {
                    self.offset_y -= event.wheel_delta.to_point().y;
                    self.offset_x -= event.wheel_delta.to_point().x;
                }
            }
            Event::KeyDown(event) if self.is_focused => {
                match event.key {
                    Key::ArrowLeft => {
                        self.offset_x += 10.0;
                    }
                    Key::ArrowRight => {
                        self.offset_x -= 10.0;
                    }
                    Key::ArrowDown => {
                        self.offset_y -= 10.0;
                    }
                    Key::ArrowUp => {
                        self.offset_y += 10.0;
                    }
                    Key::Enter => {
                        ctx.set_handled();
                    }
                    _ => {
                    }
                }
            }
            Event::Timer(event) => {
                if let Some(token) = self.double_click_timer {
                    if token == *event && self.double_click {
                        if let Some(idx) = self.does_point_collide(self.last_click_point.unwrap()) {
                            self.set_active_node(idx);
                            self.is_focused = false;
                            self.node_editor.title_text = self.nodes.get(&idx).unwrap().label.clone();
                            self.node_editor.is_visible = true;
                            ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Auto));
                            ctx.request_update();
                        }
                        println!("double clicked!");
                    } else if token == *event {
                        if let Some(idx) = self.does_point_collide(self.last_click_point.unwrap()) {
                            self.add_node(idx, format!("Label {}", self.node_idx_count), None);
                            ctx.children_changed();
                        }
                    }
                }
                self.double_click_timer = None;
                self.double_click = false;
            }
            Event::Notification(note) if note.is(TAKEN_FOCUS) => {
                self.is_focused = false;
                ctx.submit_command(TAKE_FOCUS);
                ctx.set_handled();
            }
            Event::Notification(note) if note.is(SUBMIT_CHANGES) => {
                let idx = self.get_active_node();
                println!("update text to {}", self.node_editor.title_text.clone());
                self.nodes.get_mut(&idx.unwrap()).unwrap().label = self.node_editor.title_text.clone();
                self.node_editor.is_visible = false;
                self.is_focused = true;
                ctx.request_layout();
            }
            _ => ()
        }
        if self.node_editor.is_visible {
            self.node_editor.container.event(ctx, event, &mut self.node_editor.title_text, _env);
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &(), _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.children_changed();
                ctx.request_anim_frame();
                self.update_node_coords();
            }
            _ => ()
        }
        self.node_editor.container.lifecycle(ctx, event, &self.node_editor.title_text, _env);
    }
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {
        self.node_editor.container.update(ctx, &self.node_editor.title_text, _env);
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data).unwrap();
                let layout = ctx.text().new_text_layout(node.label.clone())
                    .font(FontFamily::SANS_SERIF, 24.0)
                    .text_color(Color::BLACK)
                    .build()
                    .unwrap();
                node.container.layout = Some(layout);
        });

        //Layout editor
        if let Some(idx) = self.get_active_node() {

            let mut ne_bc = BoxConstraints::new(Size::new(0., 0.), Size::new(0., 0.));
            if self.node_editor.is_visible {
                ne_bc = BoxConstraints::new(Size::new(0., 0.), Size::new(1024., 1024.));
            }
            self.node_editor.container.layout(ctx, &ne_bc, &self.node_editor.title_text, _env);
            let node = self.nodes.get(&idx).unwrap();
            let size = node.container.layout.as_ref().unwrap().size().clone();
            let bottom_left = Point::new(node.pos.x-(size.width/2.), node.pos.y+(size.height/2.)+DEFAULT_BORDER_SIZE);
            self.node_editor.container.set_origin(ctx, &"".to_string(), _env, self.translate*bottom_left);
        }

        return bc.max();
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let vec = ctx.size();
        self.translate = TranslateScale::new((vec.to_vec2()/2.0)+Vec2::new(self.offset_x, self.offset_y), 1.0);
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::WHITE);

        //Draw edges
        self.graph.visit_edges(|node1, node2, _edge| {
            let p0 = Point::new(node1.x() as f64, node1.y() as f64);
            let p1 = Point::new(node2.x() as f64, node2.y() as f64);
            let path = Line::new(p0, p1);
            ctx.with_save(|ctx| {
                ctx.transform(Affine::from(self.translate));
                ctx.transform(Affine::from(self.scale));
                ctx.stroke(path, &Color::SILVER, DEFAULT_EDGE_WIDTH);
            });
        });

        //Draw nodes
        self.graph.visit_nodes(|node| {
            let node = self.nodes.get_mut(&node.data.user_data).unwrap();
            ctx.with_save(|ctx| {
                // if root is 0,0 translate to place that in center
                let label_size = node.container.layout.as_mut().unwrap().size();
                ctx.transform(Affine::from(self.translate));
                ctx.transform(Affine::from(self.scale));
                ctx.transform(Affine::from(TranslateScale::new(-1.0*(label_size.to_vec2())/2.0, 1.0)));
                ctx.transform(Affine::from(TranslateScale::new(node.pos, 1.0)));
                let rect = label_size.to_rect().inflate(DEFAULT_BORDER_SIZE, DEFAULT_BORDER_SIZE);
                let border = druid::piet::kurbo::RoundedRect::from_rect(rect, 5.0);
                let mut border_color = Color::BLACK;
                if node.is_active {
                    border_color = Color::RED;
                }
                ctx.fill(border, &Color::grey8(200));
                ctx.stroke(border, &border_color, DEFAULT_BORDER_SIZE);
                ctx.draw_text(node.container.layout.as_mut().unwrap(), Point::new(0.0, 0.0));
            });
        });

        //Paint editor dialog
        if self.node_editor.is_visible {
            if let Some(_idx) = self.get_active_node() {
                ctx.with_save(|ctx| {
                    self.node_editor.container.paint(ctx, &self.node_editor.title_text, _env);
                });
            }
        }

        //Draw click events and collision rects
        if DEBUG_SHOW_EVENT_VISUALS {
            if let Some(lcp) = self.last_click_point {
                ctx.fill(Circle::new(lcp, 5.0), &Color::RED);
            }

            self.last_collision_rects.iter().for_each(|r| {
                ctx.stroke(r, &Color::RED, 3.0);
            });
        }
    }
}

pub fn main() {
    let mapper = VimMapper::new();
    let window = WindowDesc::new(|| mapper).title("VimMapper").set_window_state(WindowState::MAXIMIZED);
    AppLauncher::with_window(window)
    .use_simple_logger()
    // .delegate(delegate)
    .launch(())
    .expect("launch failed");
}