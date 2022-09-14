use druid::kurbo::{Line};
use druid::piet::{ Text, TextLayoutBuilder, TextLayout, D2DTextLayout};
use force_graph::{ForceGraph, NodeData, DefaultNodeIdx, EdgeData, SimulationParameters};
use druid::widget::{prelude::*};
use druid::{AppLauncher, Color, WindowDesc, FontFamily, Affine, Point, Selector};
use std::collections::HashMap;

pub const ADD_NODE: Selector<u16> = Selector::new("add_node");
pub const BORDER_SIZE: f64 = 5.0;
struct VimMapper {
    graph: ForceGraph<u16, u16>,
    animating: bool,
    nodes: HashMap<u16, VMNode>,
    edges: HashMap<u16, VMEdge>,
    node_idx_count: u16,
    edge_idx_count: u16,
}

struct VMNodeLayoutContainer {
    layout: Option<D2DTextLayout>,
    parent: Option<u16>, 
    x: f64,
    y: f64,
    #[allow(dead_code)]
    index: u16,
}

impl<'a> VMNodeLayoutContainer {
    pub fn new(_label: String, index: u16) -> VMNodeLayoutContainer {
        let new_layout = VMNodeLayoutContainer {
            layout: None,
            parent: None,
            x: 0.0,
            y:0.0,
            index: index,
        };
        // new_layout.layout.set_font(
        //             FontDescriptor {
        //                 family: FontFamily::SANS_SERIF,
        //                 size: 24.0,
        //                 weight: FontWeight::REGULAR,
        //                 style: druid::FontStyle::Regular,
        //             }
        // );
        // new_layout.layout.set_text_color(Color::rgb8(0,0,0));
        // new_layout.layout.set_text(label.clone());
        new_layout
    }

    pub fn set_coords(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
    #[allow(dead_code)]
    pub fn set_parent(&mut self, parent: u16) {
        self.parent = Some(parent)
    }
}
struct VMNode {
    label: String,
    edges: Vec<u16>,
    index: u16,
    fg_index: Option<DefaultNodeIdx>,
    x: f64,
    y: f64,
    container: VMNodeLayoutContainer,
}

#[derive(Default)]
#[allow(dead_code)]
struct VMEdge {
    label: Option<String>,
    from: u16,
    to: u16,
    index: u16,
}

impl<'a> VimMapper {
    pub fn new() -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(
            SimulationParameters {
                force_charge: 4000.0,
                force_spring: 0.5,
                force_max: 280.0,
                node_speed: 7000.0,
                damping_factor: 0.50
            }
        );
        let mut root_node = VMNode {
            label: "Root".to_string(),
            edges: Vec::with_capacity(10),
            index: 0,
            fg_index: None,
            x: 0.0,
            y: 0.0,
            container: VMNodeLayoutContainer::new("Root".to_string(), 0),
        };
        root_node.fg_index = Some(graph.add_node(NodeData { x: 0.0, y: 0.0, is_anchor: true, user_data: 0, ..Default::default() }));
        let mut mapper = VimMapper {
            graph: graph, 
            animating: true,
            nodes: HashMap::with_capacity(50),
            edges: HashMap::with_capacity(100),
            node_idx_count: 1,
            edge_idx_count: 1,
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
                    x: from_node.x + x_offset,
                    y: from_node.y + y_offset,
                    container: VMNodeLayoutContainer::new(node_label.clone(), new_idx),
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
                    x: new_node.x as f32,
                    y: new_node.y as f32,
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
                    node.x = fg_node.x() as f64;
                    node.y = fg_node.y() as f64;
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
            Event::MouseDown(event) => {

                let mut add_to_index: u16 = 0;
                self.nodes.iter().for_each(|item| {
                    let node = item.1;
                    let size = node.container.layout.as_ref().unwrap().size();
                    let mut rect = size.to_rect().inflate(BORDER_SIZE, BORDER_SIZE);
                    rect.x0 += BORDER_SIZE;
                    rect.x1 += BORDER_SIZE;
                    rect.y0 += BORDER_SIZE;
                    rect.y1 += BORDER_SIZE;
                    let mut point = event.pos;
                    point.x = point.x-node.x+(rect.size().width/2.0);
                    point.y = point.y-node.y+(rect.size().height/2.0);
                    if rect.contains(point) {
                        add_to_index = node.index;
                    }
                });

                self.add_node(add_to_index, format!("Label {}", self.node_idx_count), None);
                ctx.children_changed();
            }
            Event::Notification(note) => {
                if note.is(Selector::<u16>::new("add_node")) {
                    println!("adding from notification");
                    ctx.set_handled();
                    let index = note.get(ADD_NODE).unwrap();
                    self.add_node(*index, format!("Label {}", self.node_idx_count), None);
                    ctx.children_changed();
                }
            }
            _ => ()
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &(), _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                println!("main widget recvd WidgetAdded");
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

            node.container.set_coords(node.x, node.y); 
        });
        return bc.max();
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::WHITE);

        //Set anchor node to position half
        self.graph.visit_nodes_mut(|node| {
            if node.data.is_anchor {
                node.data.x = (rect.x1/2.0) as f32;
                node.data.y = (rect.y1/2.0) as f32;
            }
        });

        //Draw edges
        self.graph.visit_edges(|node1, node2, _edge| {
           let path = Line::new((node1.x() as f64, node1.y() as f64), (node2.x() as f64, node2.y() as f64)); 
           ctx.stroke(path, &Color::rgb8(0, 0, 0), 3.0);
        });

        //Draw nodes
        self.graph.visit_nodes(|node| {
            let node = self.nodes.get_mut(&node.data.user_data).unwrap();
            ctx.with_save(|ctx| {
                // if !rect.contains(Point::new(node.x, node.y)) {
                //     println!("Graph has overflowed");
                // }
                let size = node.container.layout.as_mut().unwrap().size();
                let rect = size.to_rect().inflate(BORDER_SIZE, BORDER_SIZE);
                let border = druid::piet::kurbo::RoundedRect::from_rect(rect, 5.0);
                ctx.transform(Affine::translate((node.x - (size.width/2.0), node.y - (size.height/2.0))));
                ctx.fill(border, &Color::grey8(200));
                ctx.stroke(border, &Color::BLACK, BORDER_SIZE);
                // node.text_widget.inner.paint_raw(ctx, _data, _env);
                ctx.draw_text(node.container.layout.as_mut().unwrap(), Point::new(0.0, 0.0));
            });
        });
    }
}

pub fn main() {
    let window = WindowDesc::new(|| VimMapper::new()).title("VimMapper");
    AppLauncher::with_window(window).use_simple_logger().launch(()).expect("launch failed");
}