// Copyright 2022 Doug Powers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use druid::keyboard_types::Key;
use druid::kurbo::{Line, TranslateScale, Circle};
use druid::piet::{ Text, TextLayoutBuilder, TextLayout};
use druid::piet::PietTextLayout;
use force_graph::{ForceGraph, NodeData, EdgeData, DefaultNodeIdx};
#[allow(unused_imports)]
use druid::widget::{prelude::*, SvgData, Svg};
use druid::{Color, FontFamily, Affine, Point, Vec2, Rect, TimerToken, Command, Target};
use std::collections::HashMap;
use std::str::SplitWhitespace;

use crate::vmnode::{VMEdge, VMNode, VMNodeEditor, VMNodeLayoutContainer};

use crate::constants::*;

use crate::vmconfig::*;

use serde::Serialize;
use serde::Deserialize;

//VimMapper is the controller class for the graph implementation and UI. 
pub struct VimMapper {
    //The ForceGraph is contained as a background object, shadowed by the the nodes and edges HashMaps.
    // The user_data structures provided are populated by the u16 index to the corresponding nodes and edges
    // in the global HashMaps. This inefficiency will be rectified in future versions of Vim-Mapper by 
    // forking force_graph and implementing a trait-based interface that will bind directly to the 
    // global nodes.
    pub graph: ForceGraph<u16, u16>,
    //A boolean that determines if, when an AnimFrame is received, whether another is requested.
    // ForceGraph and global HashMaps are only updated regularly when this value is true.
    animating: bool,
    //The global map of nodes. All references to nodes use this u16 key to avoid holding references
    // in structs.
    pub nodes: HashMap<u16, VMNode>,
    //The global map of edges. All references to edges use this u16 key to avoid holding references
    // in structs.
    pub edges: HashMap<u16, VMEdge>,
    //The global index count that provides new nodes with a unique u16 key.
    node_idx_count: u16,
    //The global index count that provides new edges with a unique u16 key.
    edge_idx_count: u16,
    //The translate portion of the canvas transform. This pans the canvas. Updated only during paints.
    pub translate: TranslateScale,
    //The scale portion of the canvas transform. This zooms the canvas. These two transforms are
    // kept separate to allow various vectors to be scaled without translation or vice versa. Updated
    // only during paints.
    pub scale: TranslateScale,
    //Constantly updated value for x panning. Is initialized using the DEFAULT_OFFSET_X constant. All
    // events which affect panning modify this value. It is used to build the translate TranslateScale
    // during painting.
    offset_x: f64,
    //Constantly updated value for y panning. Is initialized using the DEFAULT_OFFSET_Y constant. All
    // events which affect panning modify this value. It is used to build the translate TranslateScale
    // during painting.
    offset_y: f64,
    //This holds the last location the user clicked in order to determine double clicks 
    last_click_point: Option<Point>,
    //This is a debug vector containing all the node collision rects from the last click interaction.
    last_collision_rects: Vec<Rect>,
    //This bool allows Vim-Mapper to determine if the sheet or VMNodeEditor has focus. Notifications
    // and Commands are used to pass focus between the two.
    is_focused: bool,
    //An Option that holds the global edge index corresponding to the target edge. This edge will be 
    // traversed with the Enter keybind to reach the target node.
    pub target_edge: Option<u16>,
    //A struct that holds state and widgets for the modal node editor.
    node_editor: VMNodeEditor,
    //A bool that specifies whether or not a MouseUp event has been received. If not, MouseMoves will 
    // pan the canvas.
    is_dragging: bool,
    //The point at which the last MouseDown was received. This is used to create a Vec2 that can be
    // applied to the translate TranslateScale.
    drag_point: Option<Point>,
    //The timer that, when expired, determines that the use submitted two distinct clicks rather than
    // a double click. Duration is the DOUBLE_CLICK_THRESHOLD constant.
    double_click_timer: Option<TimerToken>,
    //This value is true until the double_click_timer has passed the DOUBLE_CLICK_THRESHOLD and signals
    // that the subsequent click should be interpreted as a double click.
    double_click: bool,
    //This tuple captures the state of canvas translation so that all MouseMove deltas can be accumulated
    // to compute panning
    translate_at_drag: Option<(f64, f64)>,
    //This captures the is_hot context value during lifecycle changes to allow for the VimCanvas widget
    // to isolate click events for the dialog widgets
    is_hot: bool,
    //Toggle to display data from the VimMapper struct on-screen. (Alt-F12)
    pub debug_data: bool,
    //Toggle to display various debug visuals, including the last collision and click events as well
    // as the system palette colors in the Environment
    pub debug_visuals: bool,
    //Stores the largest individual movement (in either x or y) of any nodes during an update.
    // Used to pause computation once the graph has stabilized. 
    largest_node_movement: Option<f64>,

    //Set when a composing command (currently only mark select) is pressed. Default duration is 
    // DEFAULT_COMPOSE_TIMEOUT
    compose_select_timer: Option<TimerToken>,
    //Set to Some(<firing char>) when a composing command is started and has not yet timed out.
    // Set to None otherwise.
    compose_select: Option<String>,
    // Cached dimensions of the screen. Used to compute the offsets required to scroll a given
    // Rect into view.
    canvas_rect: Option<Rect>,
    // Struct to hold persistent VMConfig struct.
    pub config: VMConfig,
}

//A boiled-down struct to hold the essential data to serialize and deserialize a graph sheet. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Serialize, Deserialize)]
pub struct VMSave {
    nodes: HashMap<u16, BareNode>,
    edges: HashMap<u16, BareEdge>,
    node_idx_count: u16,
    edge_idx_count: u16,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

//A boiled-down struct to hold the essential data to serialize and deserialize a node. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Serialize, Deserialize)]
pub struct BareNode {
    label: String,
    edges: Vec<u16>,
    index: u16,
    pos: (f64, f64),
    is_active: bool,
    mark: Option<String>,
    targeted_internal_edge_idx: Option<usize>,
    mass: f32,
    anchored: bool,
}

impl Default for BareNode {
    fn default() -> Self {
        BareNode { 
            label: "New label".to_string(),
            edges: vec![0], 
            index: 0, 
            pos: (0.,0.), 
            is_active: false, 
            mark: None, 
            targeted_internal_edge_idx: None, 
            mass: DEFAULT_NODE_MASS, 
            anchored: false 
        }
    }
}

//A boiled-down struct to hold the essential data to serialize and deserialize an edge. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Serialize, Deserialize)]
pub struct BareEdge {
    label: Option<String>,
    from: u16,
    to: u16,
    index: u16,
}

impl VimMapper {
    pub fn new(config: VMConfig) -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(
            DEFAULT_SIMULATION_PARAMTERS
        );
        //The default node. Is always at index 0 and position (0.0, 0.0).
        let mut root_node = VMNode {
            label: DEFAULT_ROOT_LABEL.to_string(),
            edges: Vec::with_capacity(10),
            index: 0,
            pos: Vec2::new(0.0, 0.0),
            container: VMNodeLayoutContainer::new(0),
            is_active: true,
            ..Default::default()
        };
        // Capture the DefaultNodeIdx and store it in the VMNode. This allows nodes to refer to the 
        // bare ForceGraph to remove themselves.
        root_node.fg_index = Some(graph.add_node(NodeData 
            { x: 0.0, 
            y: 0.0, 
            is_anchor: true, 
            user_data: 0, 
            mass: DEFAULT_NODE_MASS, 
            ..Default::default() 
        }));
        let mut mapper = VimMapper {
            graph: graph, 
            animating: true,
            nodes: HashMap::with_capacity(50),
            edges: HashMap::with_capacity(100),
            //Account for the already-added root node
            node_idx_count: 1,
            edge_idx_count: 0,
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
            target_edge: None,
            translate_at_drag: None,
            double_click_timer: None,
            double_click: false,
            is_hot: true,
            debug_data: false,
            debug_visuals: false,
            largest_node_movement: None,
            compose_select_timer: None,
            compose_select: None,
            canvas_rect: None,
            config,
        };
        mapper.nodes.insert(0, root_node);
        mapper
    }

    //Instantiates a new VimMapper struct from a deserialized VMSave. The ForceGraph is created from scratch
    // and no fg_index values are guaranteed to persist from session to session.
    pub fn from_save(save: VMSave, config: VMConfig) -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(DEFAULT_SIMULATION_PARAMTERS);
        let mut nodes: HashMap<u16, VMNode> = HashMap::with_capacity(50);
        let mut edges: HashMap<u16, VMEdge> = HashMap::with_capacity(100);
        for (_k ,v) in save.nodes {
            let fg_index: Option<DefaultNodeIdx>;
            if v.index == 0 {
                fg_index = Some(graph.add_node(NodeData {
                    is_anchor: true,
                    x: v.pos.0 as f32,
                    y: v.pos.1 as f32,
                    mass: v.mass,
                    user_data: {
                        0
                    },
                    ..Default::default()
                }));
            } else {
                fg_index = Some(graph.add_node(NodeData {
                    is_anchor: v.anchored,
                    x: v.pos.0 as f32,
                    y: v.pos.1 as f32,
                    mass: v.mass,
                    user_data: {
                        v.index
                    },
                    ..Default::default()
                }));
            }
            nodes.insert(v.index, VMNode {
                label: v.label.clone(), 
                edges: v.edges, 
                index: v.index, 
                fg_index: fg_index, 
                pos: Vec2::new(v.pos.0, v.pos.1), 
                container: VMNodeLayoutContainer::new(v.index), 
                mark: v.mark,
                ..Default::default()
            });
        }
        for (_k,v) in save.edges {
            graph.add_edge(
                nodes.get(&v.from).unwrap().fg_index.unwrap(), 
                nodes.get(&v.to).unwrap().fg_index.unwrap(), 
                EdgeData { user_data: v.index });
            edges.insert(v.index, VMEdge { 
                label: None, 
                from: v.from, 
                to: v.to, 
                index: v.index, 
                });
        }
        let mut vm = VimMapper {
            graph,
            animating: true,
            nodes,
            edges,
            node_idx_count: save.node_idx_count,
            edge_idx_count: save.edge_idx_count,
            translate: TranslateScale::new(
                Vec2::new(
                    save.translate.0, 
                    save.translate.1),
                0.),
            scale: TranslateScale::new(
                Vec2::new(
                    0., 
                    0.),
                save.scale),
            offset_x: save.offset_x,
            offset_y: save.offset_y,
            last_click_point: None,
            last_collision_rects: Vec::new(),
            is_focused: true,
            target_edge: None,
            node_editor: VMNodeEditor::new(),
            is_dragging: false,
            drag_point: None,
            double_click_timer: None,
            double_click: false,
            translate_at_drag: None,
            is_hot: true,
            debug_data: false,
            debug_visuals: false,
            largest_node_movement: None,
            compose_select_timer: None,
            compose_select: None,
            canvas_rect: None,
            config,
        };
        vm.set_node_as_active(0);
        vm
    }

    //Instantiates a serializable VMSave from the VimMapper struct. All ForceGraph data is discarded and
    // must be recreated when the VMSave is deserialized and instantiated into a VimMapper struct
    pub fn to_save(&self) -> VMSave {
        let mut nodes: HashMap<u16, BareNode> = HashMap::with_capacity(50);
        let mut edges: HashMap<u16, BareEdge> = HashMap::with_capacity(100);
        self.nodes.iter().for_each(|(index, node)| {
            nodes.insert(*index, BareNode {
                label: node.label.clone(),
                edges: node.edges.clone(),
                index: node.index,
                pos: (node.pos.x, node.pos.y),
                is_active: false,
                targeted_internal_edge_idx: None,
                mark: node.mark.clone(),
                mass: node.mass,
                anchored: node.anchored,
            });
        });
        self.edges.iter().for_each(|(index, edge)| {
            edges.insert(*index, BareEdge {
                label: None,
                from: edge.from,
                to: edge.to,
                index: *index,
            });
        });
        let save = VMSave {
            nodes: nodes,
            edges: edges,
            node_idx_count: self.node_idx_count,
            edge_idx_count: self.edge_idx_count,
            translate: (self.translate.as_tuple().0.x, self.translate.as_tuple().0.y),
            scale: self.scale.as_tuple().1,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
        };
        save
    }

    pub fn add_node(&mut self, from_idx: u16, node_label: String, edge_label: Option<String>) -> Option<u16> {
        //Set animating to true to allow frozen sheets to adapt to new node
        self.animating = true;
        let new_node_idx = self.increment_node_idx();
        let new_edge_idx = self.increment_edge_idx();
        let from_node = self.nodes.get_mut(&from_idx);

        //Offset the new node from its progenitor to keep the ForceGraph from applying too-great repulsion
        // forces.
        let x_offset = (rand::random::<f64>()-0.5) * 10.0;
        let y_offset = (rand::random::<f64>()-0.5) * 10.0;
        match from_node {
            //Nodes must be added from an existing node.
            Some(from_node) => {
                let mut new_node = VMNode {
                    label: node_label.clone(),
                    edges: Vec::with_capacity(10),
                    index: new_node_idx,
                    pos: Vec2::new(from_node.pos.x + x_offset, from_node.pos.y + y_offset),
                    container: VMNodeLayoutContainer::new(new_node_idx),
                    ..Default::default()
                };
                let new_edge: VMEdge;
                match edge_label {
                    Some(string) => {
                        new_edge = VMEdge {
                            label: Some(string),
                            from: from_node.index,
                            to: new_node.index,
                            index: new_edge_idx,
                        }
                    }
                    _ => {
                        new_edge = VMEdge {
                            label: None,
                            from: from_node.index,
                            to: new_node.index,
                            index: new_edge_idx
                        }
                    } 
                }
                new_node.fg_index = Some(self.graph.add_node(NodeData {
                    x: new_node.pos.x as f32,
                    y: new_node.pos.y as f32,
                    user_data: new_node.index,
                    mass: DEFAULT_NODE_MASS,
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
        Some(new_node_idx)
    }

    //Deletes a leaf node. Returns the global index of the node it was attached to. Currently only
    // nodes with a single edge (leaf nodes) can be deleted.
    // TODO: implement graph traversal to allow any node (save the root) to be deleted along with
    // its children. Will require a visual prompt for confirmation.
    pub fn delete_node(&mut self, idx: u16) -> Result<u16, String> {
        //Set animating to true to allow frozen sheets to adapt to new node
        self.animating = true;
        if idx == 0 {
            return Err("Cannot delete root node!".to_string());
        }
        if let Some(node) = self.nodes.get(&idx) {
            if node.edges.len() > 1 {
                return Err("Node is not a leaf".to_string());
            } else {
                let edge = self.edges.get(&node.edges[0]).unwrap();
                let remainder: u16;
                if idx == edge.from {
                    remainder = edge.to;
                } else {
                    remainder = edge.from;
                }
                self.graph.remove_node(node.fg_index.unwrap());
                let removed_edge = node.edges[0].clone();
                self.edges.remove(&removed_edge);
                self.nodes.remove(&idx);
                let r_node = self.nodes.get_mut(&remainder).unwrap();
                r_node.targeted_internal_edge_idx = None;
                for i in 0..r_node.edges.len().clone() {
                    if r_node.edges[i] == removed_edge {
                        r_node.edges.remove(i);
                        break;
                    }
                }
                self.target_edge = None;
                return Ok(remainder);
            }
        } else {
            return Err("Node does not exist!".to_string());
        }
    }

    //Given any two node indices, return the edge that connects the two
    #[allow(dead_code)]
    pub fn get_edge(&self, idx_1: u16, idx_2: u16) -> Option<u16> {
        let mut return_edge: Option<u16> = None;
        self.edges.iter().for_each(|(idx, edge)| {
            if edge.from == idx_1 && edge.to == idx_2 {
                return_edge = Some(*idx); 
            } else if edge.from == idx_2 && edge.to == idx_1 {
                return_edge = Some(*idx);
            }
        });
        return return_edge;
    }

    //Iterate through the node HashMap to find the active node. Only one node can be marked as active
    // at any time. Multiple active nodes is an illegal state. No active nodes is a possible (but unlikely)
    // state.
    pub fn get_active_node_idx(&self) -> Option<u16> {
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

    //Iterate through the node HashMap to set the active node. All nodes except the specified are marked
    // as inactive in the process.
    pub fn set_node_as_active(&mut self, idx: u16) {
        if let Some(active_idx) = self.get_active_node_idx() {
            //If not activating the already active node, set the target edge to the one that points
            // to the departing node

            //Sometimes the active node will be set as active. Check for this and disregard if this
            // is the case.
            if idx != active_idx {
                // //Check to see if there exists an edge between new and old nodes, invalidate target if not.
                // // Marks traversal and clicks can allow users to transition activation from nodes that are
                // // not directly connected.
                // if let Some(new_edge) = self.get_edge(active_idx, idx) {
                //     self.nodes.get_mut(&idx).unwrap().set_target_edge_to_global_idx(new_edge);
                //     self.target_edge = Some(new_edge);
                // } else {
                //     self.target_edge = None;
                // }
                let node = self.nodes.get(&idx).expect("Tried to set a non-existent node as active.");
                if node.edges.len() > 1 {
                    for edge_idx in node.edges.clone() {
                        let edge = self.edges.get(&edge_idx).expect("Tried to get a non-existent edge.");
                        if edge.from != active_idx && edge.to != active_idx {
                            self.nodes.get_mut(&idx).expect("Tried to get non-existent target node")
                            .set_target_edge_to_global_idx(edge.index);
                            self.target_edge = Some(edge.index);
                        } 
                    }
                }
            }
        }

        self.nodes.iter_mut().for_each(|item| {
            if item.1.index == idx {
                item.1.is_active = true;
            } else {
                item.1.is_active = false;
            }
        });

        // if let Some(node) = self.nodes.get(&self.get_active_node_idx().unwrap()) {
        //     if let Some(rect) = node.node_rect {
        //         self.scroll_rect_into_view(rect);
        //     }
        // }
    }

    //Iterate through the nodes HashMap until a node with the matching mark is found. Return if found.
    pub fn get_node_by_mark(&mut self, char: String) -> Option<u16> {
        let marked_node = self.nodes.iter().find(|(_, node)| {
            if let Some(mark) = &node.mark {
                if *mark == char {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });

        if let Some((idx, _)) = marked_node {
            return Some(*idx);
        }
        return None;
    }

    //Return the current node count and increment.
    pub fn increment_node_idx(&mut self) -> u16 {
        let idx = self.node_idx_count.clone();
        self.node_idx_count += 1;
        idx
    }

    //Return the current edge count and increment.
    pub fn increment_edge_idx(&mut self) -> u16 {
        let idx = self.edge_idx_count.clone();
        self.edge_idx_count += 1;
        idx
    }

    //Iterate through the ForceGraph, updating the node HashMap to reflect the new positions.
    // Calculate the stability of the graph in the process, setting self.animating to false if
    // movement falls below the ANIMATION_MOVEMENT_THRESHOLD.
    pub fn update_node_coords(&mut self) -> () {
        let mut update_largest_movement: f64 = 0.;
        self.graph.visit_nodes(|fg_node| {
            let node: Option<&mut VMNode> = self.nodes.get_mut(&fg_node.data.user_data);
            match node {
                Some(node) => {
                    //Get the largest node movement (x or y) from the current update cycle
                    if let Some(_) = self.largest_node_movement {
                        let largest_movement: f64;
                        if (node.pos.x - fg_node.x() as f64).abs() > (node.pos.y - fg_node.y() as f64).abs() {
                            largest_movement = (node.pos.x-fg_node.x() as f64).abs();
                        } else {
                            largest_movement = (node.pos.y-fg_node.y() as f64).abs();
                        }
                        if largest_movement > update_largest_movement {
                            update_largest_movement = largest_movement;
                        }
                        node.pos = Vec2::new(fg_node.x() as f64, fg_node.y() as f64);
                    } else {
                        if (node.pos.x - fg_node.x() as f64).abs() > (node.pos.y - fg_node.y() as f64).abs() {
                            self.largest_node_movement = Some((node.pos.x-fg_node.x() as f64).abs());
                        } else {
                            self.largest_node_movement = Some((node.pos.y-fg_node.y() as f64).abs());
                        }
                    }
                    //Update node mass and anchor in global node HashMap
                    node.mass = fg_node.data.mass;
                    node.anchored = fg_node.data.is_anchor;
                }
                None => {
                    panic!("Attempted to update non-existent node coords from graph")
                }
            }
        });
        //If the largest movement this cycle exceeds an arbitrary const, stop animation and recomputation
        // until there is a change in the graph structure
        self.largest_node_movement = Some(update_largest_movement);
        if self.largest_node_movement.unwrap() < ANIMATION_MOVEMENT_THRESHOLD {
            self.animating = false;
        }
    }

    pub fn invalidate_node_layouts(&mut self) {
        self.nodes.iter_mut().for_each(|(_, node)| {
            node.container.layout = None;
            node.node_rect = None;
        });
    }

    pub fn increase_node_mass(&mut self, idx: u16) {
        if let Some(node) = self.nodes.get(&idx) {
            if let Some(fg_idx) = node.fg_index {
                self.graph.visit_nodes_mut(|fg_node| {
                    if fg_node.index() == fg_idx {
                        fg_node.data.mass += DEFAULT_MASS_INCREASE_AMOUNT;
                        self.animating = true;
                        if fg_node.data.mass > DEFAULT_MASS_INCREASE_AMOUNT {
                            fg_node.data.mass = fg_node.data.mass.round();
                        }
                        println!("{:?}", fg_node.data.mass);
                    }
                });
            }
        }
    }

    pub fn decrease_node_mass(&mut self, idx: u16) {
        if let Some(node) = self.nodes.get(&idx) {
            if let Some(fg_idx) = node.fg_index {
                self.graph.visit_nodes_mut(|fg_node| {
                    if fg_node.index() == fg_idx {
                        if fg_node.data.mass > (DEFAULT_MASS_INCREASE_AMOUNT+0.1) {
                            fg_node.data.mass -= DEFAULT_MASS_INCREASE_AMOUNT;
                            fg_node.data.mass = fg_node.data.mass.round();
                            self.animating = true;
                        } else if fg_node.data.mass > ((DEFAULT_MASS_INCREASE_AMOUNT+0.1)/10.) {
                            fg_node.data.mass -= DEFAULT_MASS_INCREASE_AMOUNT/10.;
                            self.animating = true;
                        } else if fg_node.data.mass > ((DEFAULT_MASS_INCREASE_AMOUNT+0.01)/100.) {
                            fg_node.data.mass -= DEFAULT_MASS_INCREASE_AMOUNT/100.;
                            self.animating = true;
                        }
                        println!("{:?}", fg_node.data.mass);
                    }
                });
            }
        }
    }

    pub fn reset_node_mass(&mut self, idx: u16) {
        if let Some(node) = self.nodes.get(&idx) {
            if let Some(fg_idx) = node.fg_index {
                self.graph.visit_nodes_mut(|fg_node| {
                    if fg_node.index() == fg_idx {
                        fg_node.data.mass = DEFAULT_NODE_MASS;
                        self.animating = true;
                        println!("{:?}", fg_node.data.mass);
                    }
                });
            }
        }
    }

    pub fn toggle_node_anchor(&mut self, idx: u16) {
        //Only allow non-root nodes to unanchor themselves
        if idx != 0 {
            if let Some(node) = self.nodes.get(&idx) {
                if let Some(fg_idx) = node.fg_index {
                    self.graph.visit_nodes_mut(|fg_node| {
                        if fg_node.index() == fg_idx {
                            fg_node.data.is_anchor = !fg_node.data.is_anchor;
                            self.animating = true;
                            println!("{:?}", fg_node.data.is_anchor);
                        }
                    });
                }
            }
        }
    }

    //Determine of a given Point (usually a click) intersects with a node. Return that node's index if so.
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
            let border = DEFAULT_BORDER_WIDTH*self.scale.as_tuple().1;
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

    //Start tracking dragging movement. This allows for MouseMove deltas to be accumulated to calulate
    // total pan values.
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

    //Opens the editor at a given node.
    pub fn open_editor(&mut self, ctx: &mut EventCtx, idx: u16) {
        self.set_node_as_active(idx);
        self.is_focused = false;
        self.node_editor.title_text = self.nodes.get(&idx).unwrap().label.clone();
        self.node_editor.is_visible = true;
        ctx.request_layout();
        ctx.request_update();
        if let Some(rect) = self.node_editor.editor_rect {
            self.scroll_rect_into_view(rect);
        }
        ctx.submit_command(Command::new(TAKE_FOCUS, (), Target::Auto));

    }

    //Closes the editor. Allows the value to be applied or discarded.
    pub fn close_editor(&mut self, ctx: &mut EventCtx, save: bool) {
        if save {
            //Submit changes
            let idx = self.get_active_node_idx();
            self.nodes.get_mut(&idx.unwrap()).unwrap().label = self.node_editor.title_text.clone();
            self.node_editor.is_visible = false;
            self.is_focused = true;
            ctx.request_layout();
        } else {
            //Cancel changes
            self.node_editor.is_visible = false;
            self.is_focused = true;
            ctx.request_layout();
        }
    }

    //Given an edge index, determine which, if any, of the connected nodes is not the active one.
    pub fn get_non_active_node_from_edge(&self, edge_idx: u16) -> Option<u16> {
        let from = self.edges.get(&edge_idx).unwrap().from;
        let to = self.edges.get(&edge_idx).unwrap().to;
        if from == self.get_active_node_idx().unwrap() {
            return Some(self.edges.get(&edge_idx).unwrap().to);
        } else if to == self.get_active_node_idx().unwrap() {
            return Some(self.edges.get(&edge_idx).unwrap().from);
        } else {
            None
        }
    }

    //Loop over node label generation until it fits within a set of BoxConstraints. Wraps the contents
    // once and then, if it still doesn't fit, reduce the font until it does.
    pub fn build_label_layout_for_constraints(ctx: &mut LayoutCtx, text: String, bc: BoxConstraints, config: VMConfig) -> Result<PietTextLayout, String> {
        let mut layout: PietTextLayout;
        let mut font_size = DEFAULT_LABEL_FONT_SIZE;

        if let Ok(layout) = ctx.text().new_text_layout(text.clone())
        .font(FontFamily::SANS_SERIF, font_size)
        .text_color(config.get_color("label-text-color".to_string()).ok().expect("label text color not found in config"))
        .build() {
            if bc.contains(layout.size()) {
                return Ok(layout);
            }
        }

        let text = VimMapper::split_string_in_half(text);

        loop {
            if let Ok(built) = ctx.text().new_text_layout(text.clone()) 
            .font(FontFamily::SANS_SERIF, font_size)
            .text_color(config.get_color("label-text-color".to_string()).ok().expect("label text color not found in config"))
            .build() {
                layout = built;
            } else {
                return Err("Could not build layout".to_string());
            }
            if bc.contains(layout.size()) {
                return Ok(layout);
            } else {
                font_size -= 1.;
            }
        }
    }

    //Wrap a string using a linebreak
    pub fn split_string_in_half(text: String) -> String {
        let mut split: SplitWhitespace = text.split_whitespace();
        
        let mut first_line: String = "".to_string();
        let mut second_line: String= "".to_string();
        loop {
            first_line = first_line + " " + split.next().unwrap();
            if first_line.len() > text.len()/2 {
                for word in split {
                    second_line = second_line + " " + word;
                }
                break;
            }
        }
        first_line + "\n" + &second_line
    }

    //Wrap a string n times using linebreaks
    pub fn split_string_in_n(text: String, n: u16) -> String {
        let mut split: SplitWhitespace = text.split_whitespace();
        let mut n: usize = n as usize;
        let mut lines: Vec<String> = vec!();

        if split.clone().count() < n.into() {
            n = split.clone().count();
        }

        for i in 0..n {
            loop {
                if let Some(word) = split.next() {
                    if let None = lines.get(i) {
                        lines.insert(i, "".to_string());
                    }
                    lines[i] = lines[i].clone() + " " + word;
                    if lines[i].len() > text.len()/n {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // let mut full: String = "".to_string();
        let mut full: String = lines.remove(0);
        for line in lines {
            full = full + "\n" + &line;
        }

        full
    }

    pub fn scroll_rect_into_view(&mut self, rect: Rect) {
        if let Some(canvas_rect) = self.canvas_rect {
            let union_rect = canvas_rect.union(rect);
            let top_left_offset = Vec2::new(0., 0.) - Vec2::new(union_rect.x0, union_rect.y0);
            let bottom_right_offset = Vec2::new(canvas_rect.x1, canvas_rect.y1) - Vec2::new(union_rect.x1, union_rect.y1);
            if top_left_offset != VEC_ORIGIN {
                if top_left_offset.x != 0. {
                    self.offset_x += top_left_offset.x + DEFAULT_SCROLL_PADDING;
                }
                if top_left_offset.y != 0. {
                    self.offset_y += top_left_offset.y + DEFAULT_SCROLL_PADDING;
                }
            } 
            if bottom_right_offset != VEC_ORIGIN {
                if bottom_right_offset.x != 0. {
                    self.offset_x += bottom_right_offset.x - DEFAULT_SCROLL_PADDING;
                }
                if bottom_right_offset.y != 0. {
                    self.offset_y += bottom_right_offset.y - DEFAULT_SCROLL_PADDING;
                }
            }
        }
    }

    pub fn scroll_node_into_view(&mut self, idx: u16) {
        if let Some(node) = self.nodes.get(&idx) {
            if let Some(rect) = node.node_rect {
                self.scroll_rect_into_view(rect);
            }
        }
    }

    pub fn set_config(&mut self, config: VMConfig) {
        self.config = config;
    }
}

impl<'a> Widget<()> for VimMapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        //If VimMapper is Notified to take focus, ensure that it's requested
        if self.is_focused {
            ctx.request_focus();
        }
        //If the node editor is visible, pass events to it. Both events and paints must be withheld
        // for the widget to be truly hidden and uninteractable. 
        if self.node_editor.is_visible {
            self.node_editor.container.event(ctx, event, &mut self.node_editor.title_text, _env);
        }
        match event {
            Event::AnimFrame(_interval) => {
                if self.is_hot && self.animating {
                    // for _ in 0..5 {
                        self.graph.update(0.032);
                    // }
                    self.update_node_coords();
                    ctx.request_anim_frame();
                }
                ctx.request_paint();
                ctx.request_layout();
            }
            Event::MouseUp(event) if event.button.is_left() => {
                if self.is_dragging {
                    self.set_dragging(false, None);
                } else if let Some(_token) = self.double_click_timer {
                    self.double_click = true;
                } else {
                    self.double_click_timer = Some(ctx.request_timer(DOUBLE_CLICK_THRESHOLD));
                }
                ctx.request_anim_frame();
            }
            Event::MouseDown(event) if event.button.is_left() => {
                if self.does_point_collide(event.pos) == None {
                    self.set_dragging(true, Some(event.pos));
                    if !ctx.is_handled() {
                        self.is_focused = true;
                        self.node_editor.is_visible = false;
                    }
                }
                ctx.request_anim_frame();
            }
            Event::MouseDown(event) if event.button.is_right() => {
                if let Some(idx) = self.does_point_collide(event.pos) {
                    self.add_node(idx, "New label".to_string(), None);
                }
                ctx.request_anim_frame();
            }
            Event::MouseMove(event) => {
                if self.is_dragging {
                    if let Some(drag_point) = self.drag_point {
                        let delta = drag_point - event.pos;
                        self.offset_x = self.translate_at_drag.unwrap().0 - delta.x;
                        self.offset_y = self.translate_at_drag.unwrap().1 - delta.y;
                    }
                }
                ctx.request_anim_frame();
            }
            Event::Wheel(event) => {
                if event.mods.shift() {
                    self.offset_x -= event.wheel_delta.to_point().x;
                } else if event.mods.ctrl() || event.buttons.has_right() {
                    if event.wheel_delta.to_point().y < 0.0 {
                        self.scale = self.scale.clone()*TranslateScale::scale(1.25);
                    } else {
                        self.scale = self.scale.clone()*TranslateScale::scale(0.75);
                    }
                } else {
                    self.offset_y -= event.wheel_delta.to_point().y;
                    self.offset_x -= event.wheel_delta.to_point().x;
                }
                ctx.request_anim_frame();
            }
            Event::KeyDown(event) if self.is_focused && self.compose_select == None => {
                match &event.key {
                    Key::Character(char) if *char == 'h'.to_string() => {
                        self.offset_x += 10.0;
                    }
                    Key::Character(char) if *char == 'l'.to_string() => {
                        self.offset_x -= 10.0;
                    }
                    Key::Character(char) if *char == 'j'.to_string() => {
                        if event.mods.ctrl() {
                            self.scale = self.scale.clone()*TranslateScale::scale(0.75);
                        } else {
                            self.offset_y -= 10.0;
                        }
                    }
                    Key::Character(char) if *char == 'k'.to_string() => {
                        if event.mods.ctrl() {
                            self.scale = self.scale.clone()*TranslateScale::scale(1.25);
                        } else {
                            self.offset_y += 10.0;
                        }
                    }
                    Key::Character(char) if *char == 'H'.to_string() => {
                        self.offset_x += 100.0;
                    }
                    Key::Character(char) if *char == 'L'.to_string() => {
                        self.offset_x -= 100.0;
                    }
                    Key::Character(char) if *char == 'J'.to_string() => {
                        self.offset_y -= 100.0;
                    }
                    Key::Character(char) if *char == 'K'.to_string() => {
                        self.offset_y += 100.0;
                    }
                    Key::Character(char) if *char == 'G'.to_string() => {
                        self.offset_x = 0.;
                        self.offset_y = 0.;
                    }
                    Key::Character(char) if *char == "o".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            if let Some(new_idx) = self.add_node(idx, format!("New label"), None) {
                                self.open_editor(ctx, new_idx);
                            }
                        }
                    }
                    Key::Character(char) if *char == "c".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            self.open_editor(ctx, idx);
                        }
                    }
                    Key::Character(char) if *char == "n".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            if let Some(edge_idx) = self.nodes.get_mut(&idx).unwrap().cycle_target() {
                                self.target_edge = Some(edge_idx);
                                if let Some(node_idx) = self.get_non_active_node_from_edge(edge_idx) {
                                    self.scroll_node_into_view(node_idx);
                                }
                            }
                        } else {
                            self.set_node_as_active(0);
                            self.scroll_node_into_view(0);
                        }
                    }
                    Key::Character(char) if *char == "d".to_string() => {
                        if let Some(remove_idx) = self.get_active_node_idx() {
                            if let Ok(idx) = self.delete_node(remove_idx) {
                                self.set_node_as_active(idx);
                                self.scroll_node_into_view(idx);
                            }
                        }
                    }
                    Key::Character(char) if *char == "m".to_string() => {
                        self.compose_select = Some("m".to_string());
                        self.compose_select_timer = Some(ctx.request_timer(DEFAULT_COMPOSE_TIMEOUT));
                    }
                    Key::Character(char) if *char == "'".to_string() => {
                        self.compose_select = Some("'".to_string());
                        self.compose_select_timer = Some(ctx.request_timer(DEFAULT_COMPOSE_TIMEOUT));
                    }
                    Key::Character(char) if *char == " ".to_string() => {
                    }
                    Key::Enter if !self.node_editor.is_visible => {
                        if let Some(edge_idx) = self.target_edge {
                            if let Some(node_idx) = self.get_non_active_node_from_edge(edge_idx) {
                                self.set_node_as_active(node_idx);
                                self.scroll_node_into_view(node_idx);
                                ctx.set_handled();
                            }
                        }
                    }
                    Key::Character(char) if *char == "+".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            self.nodes.get(&idx).
                            expect("Active node not found in node HashMap");

                            self.increase_node_mass(idx);
                        }
                    }
                    Key::Character(char) if *char == "-".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            self.nodes.get(&idx).
                            expect("Active node not found in node HashMap");

                            self.decrease_node_mass(idx);
                        }
                    }
                    Key::Character(char) if *char == "=".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            self.nodes.get(&idx).
                            expect("Active node not found in node HashMap");

                            self.reset_node_mass(idx);
                        }
                    }
                    Key::Character(char) if *char == "@".to_string() => {
                        if let Some(idx) = self.get_active_node_idx() {
                            self.nodes.get(&idx).
                            expect("Active node not found in node HashMap");

                            self.toggle_node_anchor(idx);
                        }
                    }
                    Key::F11 if event.mods.alt() => {
                        if self.debug_visuals {
                            self.debug_visuals = false;
                        } else {
                            self.debug_visuals = true;
                        }
                    }
                    Key::F12 if event.mods.alt() => {
                        if self.debug_data {
                            self.debug_data = false;
                        } else {
                            self.debug_data = true;
                        }
                    }
                    _ => ()
                }
                ctx.request_anim_frame();
            }
            Event::KeyDown(event) if self.is_focused && self.compose_select != None => {
                let compose_key: String = self.compose_select.clone().unwrap();
                match event.key.clone() {
                    Key::Character(char) => {
                        match char {
                            _ => {
                                match compose_key.as_str() {
                                    "'" => {
                                        if let Some(idx) = self.get_node_by_mark(char) {
                                            self.set_node_as_active(idx);
                                            self.scroll_node_into_view(idx);
                                        }
                                    }
                                    "m" => {
                                        if let Some(active_idx) = self.get_active_node_idx() {
                                            //Check that a node doesn't already have this mark
                                            if let Some(holder) = self.get_node_by_mark(char.clone()) {
                                                self.nodes.get_mut(&holder).unwrap().set_mark(" ".to_string());
                                            }
                                            self.nodes.get_mut(&active_idx).unwrap().set_mark(char.clone());
                                        }
                                    }
                                    _ => ()
                                }
                            }
                        }
                        self.compose_select = None;
                        self.compose_select_timer = None;
                        ctx.request_anim_frame();
                    }
                    _ => ()
                }
            }
            Event::Timer(event) => {
                if let Some(token) = self.double_click_timer {
                    ctx.set_handled();
                    if token == *event && self.double_click {
                        if let Some(point) = self.last_click_point {
                            if let Some(idx) = self.does_point_collide(point) {
                                self.open_editor(ctx, idx);
                            }
                        }
                    } else if token == *event && !self.is_dragging {
                        if let Some(point) = self.last_click_point {
                            if let Some(idx) = self.does_point_collide(point) {
                                self.set_node_as_active(idx);
                                self.scroll_node_into_view(idx);
                            }
                        }
                    }
                    self.double_click_timer = None;
                    self.double_click = false;
                }
                if let Some(token) = self.compose_select_timer {
                    ctx.set_handled();
                    if token == *event {
                        self.compose_select = None;
                        self.compose_select_timer = None;
                    }
                }
                ctx.request_anim_frame();
            }
            Event::Notification(note) if note.is(TAKEN_FOCUS) => {
                self.is_focused = false;
                ctx.set_handled();
                ctx.request_anim_frame();
            }
            Event::Notification(note) if note.is(SUBMIT_CHANGES) => {
                self.close_editor(ctx, true);
                //Node has new label; invalidate layout
                self.nodes.get_mut(&self.get_active_node_idx().unwrap()).unwrap().container.layout = None;
                ctx.set_handled();
                ctx.request_anim_frame();
            }
            Event::Notification(note) if note.is(CANCEL_CHANGES) => {
                self.close_editor(ctx, false);
                ctx.set_handled();
                ctx.request_anim_frame();
            }
            Event::Notification(note) if note.is(TAKE_FOCUS) => {
                if !self.node_editor.is_visible {
                    self.node_editor.container.event(ctx, event, &mut self.node_editor.title_text, _env);
                }
                ctx.request_anim_frame();
            }
            Event::Command(note) if note.is(REFRESH) => {
                self.invalidate_node_layouts();
                ctx.request_layout();
                ctx.request_anim_frame();
                ctx.set_handled();
            }
            _ => {
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &(), _env: &Env) {
        self.node_editor.container.lifecycle(ctx, event, &self.node_editor.title_text, _env);
        match event {
            LifeCycle::WidgetAdded => {
                //Register children with druid
                ctx.children_changed();
                //Kick off animation and calculation
                ctx.request_anim_frame();
            }
            LifeCycle::HotChanged(is_hot) => {
                //Cache is_hot values
                self.is_hot = *is_hot;
                self.set_dragging(false, None);
            }
            _ => {
            }
        }
    }
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {
        //Pass any updates to children
        self.node_editor.container.update(ctx, &self.node_editor.title_text, _env);
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        if let Some(rect) = self.canvas_rect {
            let vec = rect.size();
            self.translate = TranslateScale::new((vec.to_vec2()/2.0)+Vec2::new(self.offset_x, self.offset_y), 1.0);
        }
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data).unwrap();
                //Layout node label. Use cached version if available
                if let Some(_) = node.container.layout {
                } else {
                    if let Ok(layout) = VimMapper::build_label_layout_for_constraints(
                        ctx, node.label.clone(), BoxConstraints::new(
                            Size::new(0., 0.),
                            Size::new(NODE_LABEL_MAX_CONSTRAINTS.0, NODE_LABEL_MAX_CONSTRAINTS.1)
                        ),
                        self.config.clone()
                    ) {
                        node.container.layout = Some(layout.clone());
                    } else {
                        panic!("Could not build an appropriate sized label for node {:?}", node);
                    }
                }

        });

        //Layout editor
        let ne_bc = BoxConstraints::new(Size::new(0., 0.), Size::new(200., 200.));
        self.node_editor.container.layout(ctx, &ne_bc, &self.node_editor.title_text, _env);
        if let Some(idx) = self.get_active_node_idx() {
            let node = self.nodes.get(&idx).unwrap();
            let size = node.container.layout.as_ref().unwrap().size().clone();
            let bottom_left = Point::new(node.pos.x-(size.width/2.), node.pos.y+(size.height/2.)+DEFAULT_BORDER_WIDTH);
            self.node_editor.container.set_origin(ctx, &self.node_editor.title_text, _env, self.translate*self.scale*bottom_left);
        } else {
            self.node_editor.container.set_origin(ctx, &self.node_editor.title_text, _env, Point::new(0., 0.));
        }
        self.node_editor.editor_rect = Some(self.node_editor.container.layout_rect());

        return bc.max();
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let vec = ctx.size();
        self.translate = TranslateScale::new((vec.to_vec2()/2.0)+Vec2::new(self.offset_x, self.offset_y), 1.0);
        let ctx_size = ctx.size();
        let ctx_rect = ctx_size.to_rect();
        self.canvas_rect = Some(ctx_rect.clone());
        //Fill the canvas with background
        ctx.fill(ctx_rect, &self.config.get_color("sheet-background-color".to_string()).ok().expect("sheet background color not found"));

        //Draw click events, collision rects, and system palette
        if self.debug_visuals {
            if let Some(lcp) = self.last_click_point {
                ctx.fill(Circle::new(lcp, 5.0), &Color::RED);
            }

            self.last_collision_rects.iter().for_each(|r| {
                ctx.stroke(r, &Color::RED, 3.0);
            });

            let mut env_consts = _env.get_all();

            //Draw swatches of the system palette. NOTE: This palette will probably not be used for
            // Vim-Mapper coloring
            let mut x = 10.;
            let mut y = 10.;
            while let Some(item) = env_consts.next() {
                match item.1 {
                    druid::Value::Color(color) => {
                        ctx.fill(Rect::new(x, y, x+50., y+25.), color);
                        let layout = ctx.text().new_text_layout(format!("{:?}", item.0)).build().unwrap();
                        ctx.draw_text(&layout, Point::new(x+60., y));
                        if (y+35.) > ctx_size.height {
                            x += 60.;
                            y = 10.;
                        } else {
                            y += 35.;
                        }
                    }
                    _ => ()
                }
            }
        }

        //Draw edges
        self.graph.visit_edges(|node1, node2, _edge| {
            let p0 = Point::new(node1.x() as f64, node1.y() as f64);
            let p1 = Point::new(node2.x() as f64, node2.y() as f64);
            let path = Line::new(p0, p1);
            ctx.with_save(|ctx| {
                ctx.transform(Affine::from(self.translate));
                ctx.transform(Affine::from(self.scale));
                ctx.stroke(path, &self.config.get_color("edge-color".to_string()).ok().expect("edge color not found in config"), DEFAULT_EDGE_WIDTH);
                //If debug_data is enabled, display edge indices halfway along the edge
                if self.debug_data {
                    //Translate half-way along edge
                    let lerp = p0.lerp(p1, 0.5);
                    ctx.transform(Affine::from(TranslateScale::new(lerp.to_vec2(), 1.)));
                    let index_debug_decal = ctx.text().new_text_layout(_edge.user_data.to_string()).font(FontFamily::SANS_SERIF, 10.).text_color(Color::RED).build();
                    ctx.draw_text(&index_debug_decal.unwrap(), Point::new(0., 0.));
                }
            });
        });

        //Determine target node for painting
        let mut target_node: Option<u16> = None;
        if let Some(edge_idx) = self.target_edge {
            if let Some(node_idx) = self.get_non_active_node_from_edge(edge_idx) {
                target_node = Some(node_idx);
            }
        }

        //Draw nodes
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data)
            .expect("Expected non-option node in paint loop.");
            node.paint_node(ctx, 
                &self.config, 
                target_node, 
                &self.translate, 
                &self.scale, 
                self.debug_data); 
        });

        //Paint editor dialog
        if self.node_editor.is_visible {
            if let Some(_idx) = self.get_active_node_idx() {
                self.node_editor.container.paint(ctx, &self.node_editor.title_text, _env);
            }
        }

        //Paint compose key indicator
        if let Some(char) = self.compose_select.clone() {
            let layout = ctx.text().new_text_layout(char)
            .font(FontFamily::SANS_SERIF, DEFAULT_COMPOSE_INDICATOR_FONT_SIZE)
            .text_color( self.config.get_color("compose-indicator-text-color".to_string()).ok().expect("compose indicator text color not found in config"))
            .build().unwrap();
            ctx.draw_text(&layout, 
                (Point::new(0., ctx_size.height-layout.size().height).to_vec2() + DEFAULT_COMPOSE_INDICATOR_INSET).to_point()
                // (Point::new(0., size.height-layout.size().height).to_vec2()).to_point()
            );
        }

        //Paint debug dump
        if self.debug_data {
            if let Some(node_idx) = self.get_active_node_idx() {
                let node = self.nodes.get(&node_idx).unwrap();
                let node_edge: Option<&VMEdge>;
                if let Some(internal_idx) = node.targeted_internal_edge_idx {
                    if let Some(node_edge_idx) = node.edges.get(internal_idx) {
                        node_edge = self.edges.get(node_edge_idx);
                    } else {
                        node_edge = None;
                    }
                } else {
                    node_edge = None;
                }
                let system_edge: Option<&VMEdge>;
                if let Some(target) = self.target_edge {
                    if let Some(edge) = self.edges.get(&target) {
                        system_edge = Some(edge);
                    } else {
                        system_edge = None;
                    }
                } else {
                    system_edge = None;
                }
                let text = format!(
                        "Is Animating: {:?}\nLarget Node Movement: {:?}\nActive Node:{:?}\nNode Target: {:?}\n System Target: {:?}", 
                        self.animating,
                        self.largest_node_movement,
                        self.get_active_node_idx(),
                        VimMapper::split_string_in_n(format!("{:?}", node_edge), 2),
                        VimMapper::split_string_in_n(format!("{:?}", system_edge), 2),
                );
                let layout = ctx.text().new_text_layout(text)
                    .font(FontFamily::SANS_SERIF, 16.)
                    .text_color(Color::RED)
                    .build();

                if let Ok(text) = layout {
                    ctx.with_save(|ctx| {
                        let canvas_size = ctx.size();
                        let layout_size = text.size();
                        let point = Point::new(canvas_size.width-layout_size.width-50., canvas_size.height-layout_size.height-50.);
                        ctx.draw_text(&text, point);
                    });
                }
            }
        }
    }
}

