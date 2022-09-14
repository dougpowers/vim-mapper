use druid::kurbo::{Line};
use force_graph::{ForceGraph, NodeData, DefaultNodeIdx, EdgeData};
use druid::widget::{prelude::*, Label, Container};
use druid::{AppLauncher, Color, WindowDesc, FontFamily, Affine, WidgetPod, Point, WidgetExt, FontDescriptor, FontWeight, Selector};
use std::collections::HashMap;

pub const ADD_NODE: Selector<u16> = Selector::new("add_node");
struct VimMapper {
    graph: ForceGraph<u16, u16>,
    animating: bool,
    nodes: HashMap<u16, VMNode>,
    edges: HashMap<u16, VMEdge>,
    node_idx_count: u16,
    edge_idx_count: u16,
}

struct VMNodeWidget {
    inner: WidgetPod<(), Container<()>>,
    parent: Option<u16>, 
    x: f64,
    y: f64,
    index: u16,
}

impl<'a> VMNodeWidget {
    pub fn new(label: String, index: u16) -> VMNodeWidget {
        let widget = VMNodeWidget {
            inner: WidgetPod::new(
                Label::new(label.clone())
                    .with_font(FontDescriptor {
                        family: FontFamily::SANS_SERIF,
                        size: 24.0,
                        weight: FontWeight::REGULAR,
                        style: druid::FontStyle::Regular,
                    })
                    .with_text_color(Color::rgb8(0,0,0))
                    .border(Color::rgb8(0,0,0), 2.0)
                    .background(Color::grey(0.8))
                    .rounded(3.0)
            ),
            parent: None,
            x: 0.0,
            y:0.0,
            index: index,
        };
        widget
    }

    pub fn set_origin(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
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
    text_widget: WidgetPod<(), VMNodeWidget>,
}

#[derive(Default)]
#[allow(dead_code)]
struct VMEdge {
    label: Option<String>,
    from: u16,
    to: u16,
    index: u16,
}

impl<'a> Widget<()> for VMNodeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
        self.inner.event(ctx, event, data, env);
        if self.inner.is_hot() {
            match event {
                Event::MouseDown(_) => {
                    println!("component clicked");
                    ctx.set_handled();
                    ctx.submit_notification(ADD_NODE.with(self.index));
                }
                _ => ()
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(), env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
        match event {
            LifeCycle::WidgetAdded => {
                println!("WidgetAdded for {:?}, {:?}", ctx.widget_id(), self.inner.widget().id());
                // ctx.children_changed();
            }
            _ => ()
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &(), env: &Env) -> Size {
        let inner_bc = BoxConstraints::new(Size::new(0.0,0.0), Size::new(150.0, 50.0));
        // self.inner.widget_mut().layout(ctx, &inner_bc, data, env);
        self.inner.layout(ctx, &inner_bc, data, env).to_rect();
        self.inner.set_origin(ctx, data, env, Point::new(0.0, 0.0));
        inner_bc.max()
    }
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(), data: &(), env: &Env) {
    //    self.inner.update(ctx, data, env)
        self.inner.widget_mut().update(ctx, _old_data, data, env);
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &(), env: &Env) {
       self.inner.paint(ctx, data, env);
    }
}

impl<'a> VimMapper {
    pub fn new() -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(Default::default());
        let mut root_node = VMNode {
            label: "Root".to_string(),
            edges: Vec::with_capacity(10),
            index: 0,
            fg_index: None,
            x: 0.0,
            y: 0.0,
            text_widget: WidgetPod::new(VMNodeWidget::new("Root".to_string(), 0)),
        };
        root_node.text_widget.widget_mut().set_parent(0);
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
                    text_widget: WidgetPod::new(VMNodeWidget::new(node_label.clone(), new_idx)),
                };
                new_node.text_widget.widget_mut().set_parent(new_idx);
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
                    node.text_widget.widget_mut().x = fg_node.x() as f64;
                    node.text_widget.widget_mut().y = fg_node.y() as f64;
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
        self.nodes.iter_mut().for_each(|node| {
                node.1.text_widget.event(ctx, event, _data, _env);
                // node.1.text_widget.widget_mut().event(ctx, event, _data, _env)
        });
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
            Event::MouseDown(_) => {
                self.add_node(0, format!("Label {}", self.node_idx_count), None);
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
        self.nodes.iter_mut().for_each(|node| {
            println!("lifecycle node {:?}({:?})", node.0,node.1.text_widget.id());
            // node.1.text_widget.widget_mut().inner.lifecycle(ctx, event, _data, _env);
            // node.1.text_widget.widget_mut().lifecycle(ctx, event, _data, _env);
            node.1.text_widget.lifecycle(ctx, event, _data, _env);
            if !node.1.text_widget.is_initialized() {
                println!("child {:?} is not initialized", node.1.text_widget.id());
            }
        });
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
    fn layout(&mut self, layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        let node_bc = BoxConstraints::new(Size::new(0.0, 0.0), Size::new(150.0, 50.0));
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data).unwrap();
            node.text_widget.layout(layout_ctx, bc, _data, _env);
            // node.text_widget.widget_mut().layout(layout_ctx, &node_bc, _data, _env);
            let size = node.text_widget.widget().inner.layout_rect().size();
            let point = Point::new(node.text_widget.widget().x - (size.width/2.0), node.text_widget.widget().y - (size.height/2.0));
            node.text_widget.set_origin(layout_ctx, _data, _env, point); 
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
                // ctx.transform(Affine::translate((node.x - (size.width/2.0), node.y - (size.height/2.0))));
                // node.text_widget.inner.paint_raw(ctx, _data, _env);
                node.text_widget.paint(ctx, _data, _env);
            });
        });
    }
}

pub fn main() {
    let window = WindowDesc::new(|| VimMapper::new()).title("VimMapper");
    AppLauncher::with_window(window).use_simple_logger().launch(()).expect("launch failed");
}