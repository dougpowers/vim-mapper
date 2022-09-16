use druid::keyboard_types::Key;
use druid::kurbo::{Line, TranslateScale, Circle};
use druid::piet::{ Text, TextLayoutBuilder, TextLayout, D2DTextLayout};
use force_graph::{ForceGraph, NodeData, DefaultNodeIdx, EdgeData, SimulationParameters};
use druid::widget::{prelude::*, Container, TextBox};
use druid::{AppLauncher, Color, WindowDesc, FontFamily, Affine, Point, Vec2, Rect, Selector, WindowState, WidgetPod};
use std::collections::HashMap;

pub const DEFAULT_BORDER_SIZE: f64 = 3.0;
pub const DEFAULT_EDGE_WIDTH: f64 = 3.0;

pub const ROOT_FOCUS: Selector<()> = Selector::new("request focus");

pub const DEFAULT_SIMULATION_PARAMTERS: SimulationParameters = SimulationParameters {
    force_charge: 7000.0,
    force_spring: 1.9,
    force_max: 280.0,
    node_speed: 7000.0,
    damping_factor: 0.50
};

pub const DEFAULT_TRANSLATE: TranslateScale = TranslateScale::new(
    Vec2::new(0.0, 0.0), 1.0
);

pub const DEFAULT_OFFSET_X: f64 = 0.0;
pub const DEFAULT_OFFSET_Y: f64 = 0.0;

pub const DEFAULT_SCALE: TranslateScale = TranslateScale::new(
    Vec2::new(0.0, 0.0), 1.0
);
const DEBUG_SHOW_EVENT_VISUALS: bool = false;

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
    node_editor: VMNodeEditor,
    is_dragging: bool,
    drag_point: Option<Point>,
    translate_at_drag: Option<(f64, f64)>,
}

struct VMNodeLayoutContainer {
    layout: Option<D2DTextLayout>,
    parent: Option<u16>, 
    #[allow(dead_code)]
    index: u16,
}

struct VMNode {
    label: String,
    edges: Vec<u16>,
    index: u16,
    fg_index: Option<DefaultNodeIdx>,
    pos: Vec2,
    container: VMNodeLayoutContainer,
    is_active: bool,
}

struct VMNodeEditor {
    container: WidgetPod<String, Container<String>>,
}

impl VMNodeEditor {
    pub fn new() -> VMNodeEditor {
        let padding = Container::new(TextBox::<String>::new());
        let nodeeditor = VMNodeEditor {
            container: WidgetPod::<String, Container<String>>::new(padding),
        };
        nodeeditor
    }
}

#[allow(dead_code)]
struct VMEdge {
    label: Option<String>,
    from: u16,
    to: u16,
    index: u16,
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
}

impl<'a> Widget<()> for VimMapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        if self.is_focused {
            ctx.request_focus();
        }
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
                self.is_dragging = false;
                self.drag_point = None;
                self.translate_at_drag = None;
                if event.button.is_left() {
                    self.last_collision_rects = Vec::new();
                    self.last_click_point = Some(event.pos.clone());
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
                        if rect.contains(event.pos) {
                            add_to_index = Some(node.index);
                        }
                    });

                    if let Some(idx) = add_to_index {
                        self.add_node(idx, format!("Label {}", self.node_idx_count), None);
                        ctx.children_changed();
                    }
                }
            }
            Event::MouseDown(event) => {
                if event.button.is_left() {
                    self.is_dragging = true;
                    self.drag_point = Some(event.pos);
                    self.translate_at_drag = Some((self.offset_x, self.offset_y));
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
            Event::KeyDown(event) => {
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
                    _ => ()
                }
            }
            _ => ()
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
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: & (), _env: &Env) {}
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data).unwrap();
            if let None = node.container.layout {
                let layout = ctx.text().new_text_layout(node.label.clone())
                    .font(FontFamily::SANS_SERIF, 24.0)
                    .text_color(Color::BLACK)
                    .build()
                    .unwrap();
                node.container.layout = Some(layout);
            }
        });
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
                // let point_top_left = Point::new(node.pos.x - (label_size.width/2.0), node.pos.y - (label_size.height/2.0));
                let mut border_color = Color::BLACK;
                if node.is_active {
                    border_color = Color::RED;
                }
                ctx.fill(border, &Color::grey8(200));
                ctx.stroke(border, &border_color, DEFAULT_BORDER_SIZE);
                ctx.draw_text(node.container.layout.as_mut().unwrap(), Point::new(0.0, 0.0));
            });
        });

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
    // let delegate = VMDelegate {
    //     mapper: mapper.clone(),
    // };
    let window = WindowDesc::new(|| mapper).title("VimMapper").set_window_state(WindowState::MAXIMIZED);
    AppLauncher::with_window(window)
    .use_simple_logger()
    // .delegate(delegate)
    .launch(())
    .expect("launch failed");
}