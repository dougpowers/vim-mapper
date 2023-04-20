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

use common_macros::hash_set;
use druid::kurbo::{Line, TranslateScale};
use druid::piet::{ Text, TextLayoutBuilder, TextLayout, PietText};
use druid::piet::PietTextLayout;
use vm_force_graph_rs::{ForceGraph, NodeData, EdgeData, DefaultNodeIdx};
use druid::widget::prelude::*;
use druid::{Color, FontFamily, Affine, Point, Vec2, Rect, TimerToken, Command, Target, Menu, MenuItem, FontWeight};
use regex::Regex;
use std::collections::{HashMap};
use std::f64::consts::*;

use crate::vmdialog::VMDialog;
use crate::vmgraphclip::VMGraphClip;
use crate::{vminput::*, AppState};
use crate::vmnode::VMNode;

use crate::constants::*;

use crate::vmconfig::*;

//VimMapper is the controller class for the graph implementation and UI. 

pub struct VimMapper {
    //The ForceGraph is contained as a background object, shadowed by the the nodes and edges HashMaps.
    // The user_data structures provided are populated by the u32 index to the corresponding nodes and edges
    // in the global HashMaps. This inefficiency will be rectified in future versions of Vim-Mapper by 
    // forking force_graph and implementing a trait-based interface that will bind directly to the 
    // global nodes.
    pub(crate) graph: ForceGraph<u32, u32>,
    //A boolean that determines if, when an AnimFrame is received, whether another is requested.
    // ForceGraph and global HashMaps are only updated regularly when this value is true.
    pub(crate) animating: bool,
    //The global map of nodes. All references to nodes use this u32 key to avoid holding references
    // in structs.
    pub(crate) nodes: HashMap<u32, VMNode>,
    //The global index count that provides new nodes with a unique u32 key.
    pub(crate) node_idx_count: u32,
    // //The global index count that provides new edges with a unique u32 key.
    // pub(crate) edge_idx_count: u32,
    //The translate portion of the canvas transform. This pans the canvas. Updated only during paints.
    pub(crate) translate: TranslateScale,
    //The scale portion of the canvas transform. This zooms the canvas. These two transforms are
    // kept separate to allow various vectors to be scaled without translation or vice versa. Updated
    // only during paints.
    pub(crate) scale: TranslateScale,
    //Constantly updated value for x panning. Is initialized using the DEFAULT_OFFSET_X constant. All
    // events which affect panning modify this value. It is used to build the translate TranslateScale
    // during painting.
    pub(crate) offset_x: f64,
    //Constantly updated value for y panning. Is initialized using the DEFAULT_OFFSET_Y constant. All
    // events which affect panning modify this value. It is used to build the translate TranslateScale
    // during painting.
    pub(crate) offset_y: f64,
    pub(crate) target_node_list: Vec<u32>,
    pub(crate) target_node_idx: Option<usize>,
    //This captures the is_hot context value during lifecycle changes to allow for the VimCanvas widget
    // to isolate click events for the dialog widgets
    pub(crate) is_hot: bool,
    //Toggle to display data from the VimMapper struct on-screen. (Alt-F12)
    pub(crate) debug_data: bool,
    //Toggle to display various debug visuals, including the last collision and click events as well
    // as the system palette colors in the Environment
    #[allow(dead_code)]
    pub(crate) debug_visuals: bool,
    //Stores the largest individual movement (in either x or y) of any nodes during an update.
    // Used to pause computation once the graph has stabilized. 
    pub(crate) largest_node_movement: Option<f64>,
    // Cached dimensions of the screen. Used to compute the offsets required to scroll a given
    // Rect into view.
    pub(crate) canvas_rect: Option<Rect>,
    // Struct to hold persistent VMConfig struct.
    pub(crate) config: VMConfigVersion4,
    // Whether to render non-target nodes as disabled
    pub(crate) node_render_mode: NodeRenderMode,

    pub(crate) animation_timer_token: Option<TimerToken>,

    pub(crate) last_traverse_angle: f64,

    pub(crate) enabled_layouts: HashMap<DefaultNodeIdx, PietTextLayout>,
    pub(crate) disabled_layouts: HashMap<DefaultNodeIdx, PietTextLayout>,

    pub(crate) root_nodes: HashMap<usize, DefaultNodeIdx>,

    pub(crate) input_manager: VMInputManager,

    pub(crate) last_mouse_down_data: Option<(Option<u32>, Point, (f64, f64))>,
    pub(crate) last_menu_pos: Point,
    //A value set by VMCanvas on every delegated click informing the VimMapper of how many nodes are in
    // the default register. (Used to label the context menu);
    pub(crate) default_paste_register_count: usize,
    
    pub(crate) zoom_level_index: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum NodeRenderMode {
    OnlyTargetsEnabled,
    AllEnabled,
}

impl<'a> Default for VimMapper {
    fn default() -> Self {
        let config = VMConfigVersion4::default();
        let mut graph = <ForceGraph<u32, u32>>::new(
            DEFAULT_SIMULATION_PARAMETERS
        );
        //The default node. Is always at index 0 and position (0.0, 0.0).
        let mut root_node = VMNode {
            label: DEFAULT_ROOT_LABEL.to_string(),
            index: 0,
            mark: Some("0".to_string()),
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
            //Account for the already-added root node
            node_idx_count: 1,
            translate: DEFAULT_TRANSLATE,
            scale: DEFAULT_SCALE,
            offset_x: DEFAULT_OFFSET_X,
            offset_y: DEFAULT_OFFSET_Y,
            target_node_idx: None,
            target_node_list: vec![],
            is_hot: true,
            debug_data: false,
            debug_visuals: false,
            largest_node_movement: None,
            canvas_rect: None,
            config,
            node_render_mode: NodeRenderMode::AllEnabled,
            animation_timer_token: None,
            last_traverse_angle: TAU-FRAC_PI_2,
            enabled_layouts: HashMap::new(),
            disabled_layouts: HashMap::new(),
            root_nodes: HashMap::new(),
            input_manager: VMInputManager::new(),
            last_mouse_down_data: None,
            last_menu_pos: Point::new(0.,0.),
            default_paste_register_count: 0,
            zoom_level_index: DEFAULT_ZOOM_INDEX,
        };
        let root_fg_index = root_node.fg_index.unwrap();
        mapper.nodes.insert(0, root_node);
        mapper.root_nodes.insert(mapper.graph.get_node_component(root_fg_index), root_fg_index);
        mapper
    }
}

#[allow(dead_code)]
impl<'a> VimMapper {
    pub fn new(config: VMConfigVersion4) -> VimMapper {
        let mut vm = VimMapper::default();
        vm.config = config;
        vm
    }

    pub fn get_nodes(&self) -> &HashMap<u32, VMNode> {
        return &self.nodes;
    }

    pub fn get_nodes_mut(&mut self) -> &mut HashMap<u32, VMNode> {
        return &mut self.nodes;
    }

    pub fn get_force_graph(&self) -> &ForceGraph<u32, u32> {
        return &self.graph;
    }

    pub fn get_force_graph_mut(&mut self) -> &mut ForceGraph<u32, u32> {
        return &mut self.graph;
    }

    pub fn get_offset_x(&self) -> f64 {
        return self.offset_x;
    }

    pub fn set_offset_x(&mut self, offset_x: f64) {
        self.offset_x = offset_x;
    }

    pub fn get_offset_y(&self) -> f64 {
        return self.offset_y;
    }

    pub fn set_offset_y(&mut self, offset_x: f64) {
        self.offset_y = offset_x;
    }

    pub fn set_render_mode(&mut self, mode: NodeRenderMode) {
        self.node_render_mode = mode;
    }

    #[allow(dead_code)]
    pub fn get_render_mode(&mut self) -> NodeRenderMode {
        self.node_render_mode.clone()
    }

    pub fn get_node_pos(&self, idx: u32) -> Vec2 {
        let node = self.nodes.get(&idx).expect("Tried to get position of a non-existent node");
        let fg_node = &self.graph.get_graph()[node.fg_index.unwrap()];
        return Vec2::new(fg_node.x(), fg_node.y());
    }

    pub fn get_translate(&self) -> TranslateScale {
        return self.translate;
    }

    pub fn get_scale(&self) -> TranslateScale {
        return self.scale;
    }

    pub fn get_node_idx_count(&self) -> u32 {
        return self.node_idx_count;
    }

    pub fn build_target_list_from_neighbors(&mut self, idx: u32) {
        self.target_node_list.clear();
        self.target_node_idx = None;
        let node = self.nodes.get(&idx).expect("Tried to build target list from non-existent node");
        let node_pos = self.get_node_pos(node.index);
        let mut sort_vec: Vec<(u32, Vec2, f64)> = vec![];
        let mut offsets: Vec<(usize, f64, u32)> = vec![];
        let target_angle = Vec2::from_angle(self.last_traverse_angle).normalize();
        for node_fg_idx in self.graph.get_graph().neighbors(
            node.fg_index.expect("Tried to get a non-existent fg_index from a node"))
        {
            let new_target_node_idx = self.graph.get_graph()[node_fg_idx].data.user_data;

            let target_node = self.nodes.get(&new_target_node_idx).unwrap();

            let target_node_pos = self.get_node_pos(target_node.index);
            
            let angle = Vec2::new(target_node_pos.x-node_pos.x, target_node_pos.y-node_pos.y).normalize();
            sort_vec.push((new_target_node_idx, angle, angle.atan2()));
        }
        if !sort_vec.is_empty() {
            sort_vec.sort_unstable_by(|a1, a2| {
                if a1.1.atan2() > a2.1.atan2() {
                    std::cmp::Ordering::Greater
                } else if a1.1.atan2() < a2.1.atan2() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            for i in 0..sort_vec.len() {
                offsets.push((i, (sort_vec[i].1.dot(target_angle).clamp(-1., 1.).acos()).abs(), sort_vec[i].0));
            }
            offsets.sort_unstable_by(|a1, a2| {
                if a1.1 > a2.1 {
                    std::cmp::Ordering::Greater
                } else if a1.1 < a2.1 {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            sort_vec.rotate_left(offsets[0].0);
        }
        for i in sort_vec {
            self.target_node_list.push(i.0);
        }
    }

    pub fn build_target_list_from_string(&mut self, search_string: String) -> Result<(), ()> {
        self.target_node_idx = None;
        self.target_node_list.clear();
        let regex_string = format!("(?i){}", search_string);
        let regex = Regex::new(&regex_string).expect("Failed to compile search regex");
        for (idx, node) in &self.nodes {
            if regex.is_match(&node.label) {
                if Some(*idx) != self.get_active_node_idx() {
                    self.target_node_list.push(*idx);
                }
            }
        }
        if self.target_node_list.len() > 0 {
            self.target_node_idx = Some(0);
            return Ok(());
        } else {
            return Err(());
        }
    }

    pub fn build_menu_for_node(&mut self, _ctx: &EventCtx, idx: u32) -> Menu<AppState> {
        let delete_count = self.get_node_deletion_count(idx);
        let node = self.nodes.get(&idx);
        if let Some(node) = node {
            let mut menu: Menu<AppState> = Menu::empty();
            let neighbor_count = self.graph.get_graph().neighbors(node.fg_index.unwrap()).collect::<Vec<_>>().len();
            menu = menu.entry(
                MenuItem::new("Add Child Node").command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::CreateNewNode,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global))
            );
            if self.target_node_list.contains(&idx) {
                menu = menu.entry(
                    MenuItem::new("Insert Node").command(Command::new(
                        EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::InsertNewNode,
                            index: Some(idx),
                            ..Default::default()
                        },
                        Target::Global))
                );
            }
            menu = menu.separator();
            menu = menu.entry(
                MenuItem::new(
                    "Yank Node"
                ).command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::YankNode,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global)
                )
            );
            if delete_count > 1 {
                menu = menu.entry(
                    MenuItem::new(format!("Yank {} Nodes", delete_count)).command(Command::new(
                        EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::YankNodeTree,
                            index: Some(idx),
                            ..Default::default()
                        },
                        Target::Global))
                );
            };
            menu = menu.separator();
            menu = menu.entry(
                MenuItem::new(
                    if self.default_paste_register_count > 2 {
                        format!("Paste-Append {} Nodes", self.default_paste_register_count)
                    } else if self.default_paste_register_count == 1 {
                        "Paste 1 Node".to_string()
                    } else {
                        "No Nodes to Paste".to_string()
                    }
                ).command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::PasteNodeTree,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global)).enabled(self.default_paste_register_count > 0)
            );
            menu = menu.separator();
            if neighbor_count <= 2 {
                menu = menu.entry(
                    MenuItem::new("Cut Node").command(Command::new(
                        EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::CutNode,
                            index: Some(idx),
                            ..Default::default()
                        },
                        Target::Global))
                );
            } 
            if delete_count > 1 && idx != 0 {
                menu = menu.entry(
                    MenuItem::new(format!("Cut {} Nodes", delete_count)).command(Command::new(
                        EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::AttemptNodeDeletion,
                            index: Some(idx),
                            ..Default::default()
                        },
                        Target::Global))
                );
            }
            if !(delete_count > 1 || idx != 0) && !(neighbor_count <= 2) {
                menu = menu.separator();
            }
            menu = menu.entry(
                MenuItem::new("⚓Toggle Anchor").command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::ToggleNodeAnchor,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global))
            );
            let mass_menu: Menu<AppState> = Menu::new("Mass").entry(
                MenuItem::new("Increase Mass").command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::IncreaseNodeMass,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global
                ))
            ).entry(
                MenuItem::new("Decrease Mass").command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::DecreaseNodeMass,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global
                ))
            ).entry(
                MenuItem::new("Reset Mass").command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::ResetNodeMass,
                        index: Some(idx),
                        ..Default::default()
                    },
                    Target::Global
                ))
            );
            menu = menu.entry(mass_menu);
            return menu;
        } else {
            return Menu::<AppState>::empty();
        }
    }

    pub fn build_menu_for_sheet(&mut self, _ctx: &EventCtx, pos: Option<Point>) -> Menu<AppState> {
        let mut menu: Menu<AppState> = Menu::empty();
        menu = menu.entry(
            MenuItem::new("Add External Node").command(Command::new(
                EXECUTE_ACTION,
                ActionPayload {
                    action: Action::CreateNewExternalNode,
                    pos: if let Some(point) = pos { Some(self.screen_point_to_canvas_point(point)) } else { None }, 
                    ..Default::default()
                },
                Target::Global))
        );
        menu = menu.entry(
            MenuItem::new(
                if self.default_paste_register_count > 2 {
                    format!("Paste {} Nodes as External", self.default_paste_register_count)
                } else if self.default_paste_register_count == 1 {
                    "Paste 1 Node as External".to_string()
                } else {
                    "No Nodes to Paste".to_string()
                }
            ).command(Command::new(
                EXECUTE_ACTION,
                ActionPayload {
                    action: Action::PasteNodeTreeExternal,
                    pos: if let Some(point) = pos { Some(self.screen_point_to_canvas_point(point)) } else { None }, 
                    ..Default::default()
                },
                Target::Global)).enabled(self.default_paste_register_count > 0)
        );
        if self.default_paste_register_count > 0 {
            menu = menu.entry(
                MenuItem::new(
                    if self.default_paste_register_count > 2 {
                        format!("Paste {} Nodes in New Tab", self.default_paste_register_count)
                    } else if self.default_paste_register_count == 1 {
                        "Paste 1 Node in New Tab".to_string()
                    } else {
                        "No Nodes to Paste".to_string()
                    }
                ).command(Command::new(
                    EXECUTE_ACTION,
                    ActionPayload {
                        action: Action::PasteNodeTreeAsTab,
                        ..Default::default()
                    },
                    Target::Global)).enabled(self.default_paste_register_count > 0)
            );
        }
        return menu;
    }

    pub fn target_node_if_listed(&mut self, target: u32) -> Result<(), String> {
        for (list_idx, idx) in self.target_node_list.iter().enumerate() {
            if *idx == target {
                self.target_node_idx = Some(list_idx);
                return Ok(());
            }
        }
        // return Err(String::from("specified node not in target list"));
        return Err(format!("Index {} is not in target list", target));
    }

    pub fn get_target_list_length(&self) -> usize {
        self.target_node_list.len()
    }

    pub fn cycle_target_forward(&mut self) {
        if self.target_node_idx == None && self.target_node_list.len() > 0 {
            //If not index set, set to front of list
            self.target_node_idx = Some(0);
        } else if let Some(idx) = self.target_node_idx {
            if idx == self.target_node_list.len()-1 {
                self.target_node_idx = Some(0);
            } else {
                self.target_node_idx = Some(self.target_node_idx.unwrap()+1);
            }
        }
    }

    pub fn cycle_target_backward(&mut self) {
        if self.target_node_idx == None && self.target_node_list.len() > 0 {
            //If no index set, set to back of list
            self.target_node_idx = Some(self.target_node_list.len()-1);
        } else if let Some(idx) = self.target_node_idx {
            if idx == 0 {
                self.target_node_idx = Some(self.target_node_list.len()-1);
            } else {
                self.target_node_idx = Some(self.target_node_idx.unwrap()-1);
            }
        }
    }

    pub fn get_target_node_idx(&self) -> Option<u32> {
        if let Some(idx) = self.target_node_idx {
            return Some(self.target_node_list[idx]);
        } else {
            return None;
        }
    }

    pub fn add_external_node(&mut self, node_label: String) -> Option<u32> {
        let new_node_idx = self.increment_node_idx();
        let mut new_node = VMNode {
            label: node_label.clone(),
            index: new_node_idx,
            ..Default::default()
        };
        new_node.fg_index = Some(self.graph.add_node(NodeData {
            x: 0.0,
            y: 0.0,
            user_data: new_node.index,
            mass: DEFAULT_NODE_MASS,
            is_anchor: true,
            ..Default::default()
        }));
        let component = self.graph.get_node_component(new_node.fg_index.unwrap());
        if component < 10 {
            new_node.mark = Some(component.to_string());
        }
        self.rebuild_root_nodes();
        self.root_nodes.insert(component, new_node.fg_index.unwrap());
        self.nodes.insert(new_node.index, new_node);
        self.animating = true;
        Some(new_node_idx)
    }

    pub fn add_node(&mut self, from_idx: u32, node_label: String) -> Option<u32> {
        //Set animating to true to allow frozen sheets to adapt to new node
        self.animating = true;
        let new_node_idx = self.increment_node_idx();
        // let new_edge_idx = self.increment_edge_idx();
        let from_node_pos = self.get_node_pos(from_idx);
        let from_node = self.nodes.get_mut(&from_idx);

        //Offset the new node from its progenitor to keep the ForceGraph from applying too-great repulsion
        // forces.
        let offset_vec = Vec2::new(rand::random::<f64>()-0.5, rand::random::<f64>()-0.5) * self.graph.parameters.min_attract_distance;
        let new_node_pos = Vec2::new(from_node_pos.x + offset_vec.x, from_node_pos.y + offset_vec.y);
        match from_node {
            //Nodes must be added from an existing node.
            Some(from_node) => {
                let mut new_node = VMNode {
                    label: node_label.clone(),
                    index: new_node_idx,
                    ..Default::default()
                };
                new_node.fg_index = Some(self.graph.add_node(NodeData {
                    x: new_node_pos.x,
                    y: new_node_pos.y,
                    user_data: new_node.index,
                    mass: DEFAULT_NODE_MASS,
                    ..Default::default()
                }));
                self.graph.add_edge(from_node.fg_index.unwrap(), new_node.fg_index.unwrap(), EdgeData { user_data: 0 }); 
                self.nodes.insert(new_node.index, new_node);
            }
            _ => {
                panic!("Tried to add to a non-existent node")
            } 
        }
        if let Some(idx) = self.get_active_node_idx() {
            if let Some(target_idx) = self.target_node_idx {
                let target = self.target_node_list[target_idx];
                self.build_target_list_from_neighbors(idx);
                let _ = self.target_node_if_listed(target);
            } else {
                self.build_target_list_from_neighbors(idx)
            }
        }
        Some(new_node_idx)
    }

    pub fn insert_node(&mut self, from_node_idx: u32, to_node_idx: u32, node_label: String) -> Option<u32> {
        let mut from_node: Option<DefaultNodeIdx> = None;
        let mut to_node: Option<DefaultNodeIdx> = None;
        if let Some(from_vmnode) = self.nodes.get(&from_node_idx) {
            from_node = Some(from_vmnode.fg_index.unwrap());
        }
        if let Some(to_vmnode) = self.nodes.get(&to_node_idx) {
            to_node = Some(to_vmnode.fg_index.unwrap());
        }
        if from_node.is_some() && to_node.is_some() {
            let neighbors = self.graph.get_graph().neighbors(from_node.unwrap()).collect::<Vec<_>>();
            if neighbors.contains(&to_node.unwrap()) {
                let from_node_pos = Vec2::new(self.graph.get_graph()[from_node.unwrap()].x(), self.graph.get_graph()[from_node.unwrap()].y());
                let to_node_pos = Vec2::new(self.graph.get_graph()[to_node.unwrap()].x(), self.graph.get_graph()[to_node.unwrap()].y());
                let new_node_idx = self.increment_node_idx();
                let new_node_pos = from_node_pos.lerp(to_node_pos, 0.5);
                let mut new_vmnode = VMNode {
                    label: node_label.clone(),
                    index: new_node_idx,
                    ..Default::default()
                };
                new_vmnode.fg_index = Some(self.graph.add_node(NodeData {
                    x: new_node_pos.x,
                    y: new_node_pos.y,
                    user_data: new_vmnode.index,
                    mass: DEFAULT_NODE_MASS,
                    ..Default::default()
                }));
                self.graph.add_edge(from_node.unwrap(), new_vmnode.fg_index.unwrap(), EdgeData { user_data: 0 }); 
                self.graph.add_edge(to_node.unwrap(), new_vmnode.fg_index.unwrap(), EdgeData { user_data: 0 }); 
                self.nodes.insert(new_vmnode.index, new_vmnode);
                let old_edge = self.graph.get_graph().find_edge(from_node.unwrap(), to_node.unwrap());
                self.graph.get_graph_mut().remove_edge(old_edge.unwrap());
                self.animating = true;
                self.set_node_as_active(new_node_idx);
                return Some(new_node_idx);
            }
        }
        None
    }

    pub fn get_node_deletion_count(&mut self, idx: u32) -> usize {
        if let Some(node) = self.nodes.get(&idx) {
            let node_component = self.graph.get_node_component(node.fg_index.unwrap());
            let component_root = *self.root_nodes.get(&node_component).unwrap();
            let (removal_set, _remainder) = self.graph.get_node_descendant_tree(
                self.nodes.get(&idx).unwrap().fg_index.unwrap(), component_root);
            return removal_set.len();
        } else {
            return 0;
        }
    }

    pub fn cut_node(&mut self, idx: u32, ctx: &mut EventCtx) -> Result<u32, String> {
        if let Some(node) = self.nodes.get(&idx) {
            let mut node_is_root: bool = false;
            for v in self.root_nodes.values() {
                if *v == node.fg_index.unwrap() {
                    node_is_root = true;
                }
            }
            if node_is_root {
                return Err(String::from("Cannot snip a root node."));
            }
            let neighbors = self.graph.get_graph().neighbors(node.fg_index.unwrap()).collect::<Vec<_>>();
            let neighbor_count = self.graph.get_graph().neighbors(node.fg_index.unwrap()).count();
            if neighbor_count > 2 {
                return Err(String::from("Node has more than 2 neighbors"));
            } else if neighbor_count == 2 {
                VMGraphClip::dispatch(ctx, &self, &hash_set!{node.fg_index.unwrap()}, node.fg_index.unwrap(), &"0".to_string());
                self.graph.add_edge(neighbors[0], neighbors[1], EdgeData { user_data: 0 });
                self.graph.remove_node(node.fg_index.unwrap());
                self.nodes.remove(&idx);
                self.animating = true;
                return Ok(self.graph.get_graph()[neighbors[0]].data.user_data)
            } else if neighbor_count == 1{
                self.graph.remove_node(node.fg_index.unwrap());
                self.nodes.remove(&idx);
                self.animating = true;
                return Ok(self.graph.get_graph()[neighbors[0]].data.user_data);
            } else {
                self.graph.remove_node(node.fg_index.unwrap());
                self.nodes.remove(&idx);
                self.animating = true;
                return Ok(0);
            }
        } else {
            return Err(String::from("Node not found"));
        }
    }

    pub fn rebuild_root_nodes(&mut self) {
        let mut new_roots = HashMap::new();
        for (_, v) in self.root_nodes.iter_mut() {
            let new_component = self.graph.get_node_component(*v);
            new_roots.insert(new_component, *v);
            self.nodes.get_mut(&self.graph.get_graph()[*v].data.user_data).unwrap().mark = Some(new_component.to_string());
        }
        self.root_nodes = new_roots;
    }

    //Deletes a leaf node. Returns the global index of the node it was attached to. Currently only
    // nodes with a single edge (leaf nodes) can be deleted.
    // TODO: implement graph traversal to allow any node (save the root) to be deleted along with
    // its children. Will require a visual prompt for confirmation.

    pub fn yank_node(&self, idx: u32, ctx: &mut EventCtx) -> Result<(), String> {
        if let Some(node) = &self.nodes.get(&idx) {
            let removal_list = hash_set!(node.fg_index.unwrap());
            VMGraphClip::dispatch(ctx, &self, &removal_list, node.fg_index.unwrap(), &"0".to_string());
            return Ok(());
        }
        return Err(format!("Yank of single node {} failed.", idx));
    }

    pub fn yank_node_tree(&mut self, idx: u32, ctx: &mut EventCtx) -> Result<(), String> {
        if let Some(node) = &self.nodes.get(&idx) {
            let node_component = self.graph.get_node_component(node.fg_index.unwrap());
            let component_root = *self.root_nodes.get(&node_component).unwrap();
            let (removal_list, _) = self.graph.get_node_descendant_tree(node.fg_index.unwrap(), component_root);
            VMGraphClip::dispatch(ctx, &self, &removal_list, node.fg_index.unwrap(), &"0".to_string());
            return Ok(());
        }
        return Err(format!("Yank of node tree {} failed.", idx));
    }

    pub fn cut_node_tree(&mut self, idx: u32, ctx: &mut EventCtx) -> Result<u32, String> {
        //Set animating to true to allow frozen sheets to adapt to new node
        if idx == 0 {
            return Err("Cannot delete root node!".to_string());
        }
        if let Some(node) = self.nodes.get(&idx) {
            self.animating = true;
            let node_component = self.graph.get_node_component(node.fg_index.unwrap());
            let component_root = *self.root_nodes.get(&node_component).unwrap();
            let (removal_list, remainder) = self.graph.get_node_descendant_tree(node.fg_index.unwrap(), component_root);
            VMGraphClip::dispatch(ctx, &self, &removal_list, node.fg_index.unwrap(), &"0".to_string());
            if self.is_node_root(idx) {
                for fg_idx in removal_list {
                    if fg_idx == component_root {
                        self.root_nodes.remove(&node_component);
                    }
                    self.nodes.remove(&self.graph.get_graph()[fg_idx].data.user_data);
                    self.graph.remove_node(fg_idx);
                    self.enabled_layouts.remove(&fg_idx);
                    self.disabled_layouts.remove(&fg_idx);
                }
                self.rebuild_root_nodes();
                return Ok(0);
            } else {
                let mut removal_would_unanchor_component = false;
                for idx in &removal_list {
                    if self.graph.is_sole_anchor_in_component(*idx) {
                        removal_would_unanchor_component = true;
                    }
                }
                if removal_would_unanchor_component {
                    return Err(String::from("Removal of node tree would unanchor component!"));
                } else {
                    for fg_idx in removal_list {
                        self.nodes.remove(&self.graph.get_graph()[fg_idx].data.user_data);
                        self.graph.remove_node(fg_idx);
                        self.enabled_layouts.remove(&fg_idx);
                        self.disabled_layouts.remove(&fg_idx);
                    }
                    if let Some(remainder) = remainder {
                        return Ok(self.graph.get_graph()[remainder].data.user_data);
                    } else {
                        return Ok(0)
                    }
                }
            }
        } else {
            return Err("Node does not exist!".to_string());
        }
    }

    //Iterate through the node HashMap to find the active node. Only one node can be marked as active
    // at any time. Multiple active nodes is an illegal state. No active nodes is a possible (but unlikely)
    // state.
    pub fn get_active_node_idx(&self) -> Option<u32> {
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
    pub fn set_node_as_active(&mut self, idx: u32) {
        if let Some(node) = self.get_active_node_idx() {
            let node_pos = self.get_node_pos(node);
            let target_node_pos = self.get_node_pos(idx);
            let angle = Vec2::new(target_node_pos.x-node_pos.x, target_node_pos.y-node_pos.y).atan2();
            self.last_traverse_angle = angle;
        }
        self.nodes.iter_mut().for_each(|item| {
            if item.1.index == idx {
                item.1.is_active = true;
            } else {
                item.1.is_active = false;
            }
        });
        self.build_target_list_from_neighbors(idx);
        if self.target_node_list.len() > 0 {
            self.cycle_target_forward();
        }
    }

    pub fn is_node_root(&self, idx: u32) -> bool {
        if let Some(node) = self.nodes.get(&idx) {
            for v in self.root_nodes.values() {
                if node.fg_index.unwrap() == *v {
                    return true;
                }
            }
            return false;
        } else {
            return false;
        }
    }

    pub fn set_node_mark(&mut self, idx: u32, char: String) {
        if self.is_node_root(idx) {
            return;
        }
        let regex = Regex::new("[0-9]").expect("Failed to compile regex");
        if regex.is_match(&char) {
            return;
        }
        if let Some(node) = self.nodes.get_mut(&idx) {
            if char == " " {
                node.mark = None;
            } else {
                node.mark = Some(char);
            }
        }
    }

    //Iterate through the nodes HashMap until a node with the matching mark is found. Return if found.
    pub fn get_node_by_mark(&mut self, char: String) -> Option<u32> {
        let marked_node = self.nodes.iter().find(|(_, node)| {
            if let Some(mark) = &node.mark {
                if *mark == char { true } else { false }
            } else { false }
        });

        if let Some((idx, _)) = marked_node {
            return Some(*idx);
        }
        return None;
    }

    //Return the current node count and increment.
    pub fn increment_node_idx(&mut self) -> u32 {
        let idx = self.node_idx_count.clone();
        self.node_idx_count += 1;
        idx
    }

    pub fn invalidate_node_layouts(&mut self) {
        self.nodes.iter_mut().for_each(|(_, node)| {
            self.enabled_layouts.remove(&node.fg_index.unwrap());
            self.disabled_layouts.remove(&node.fg_index.unwrap());
            node.node_rect = Rect::new(0.,0.,0.,0.);
        });
    }

    pub fn invalidate_node_layout(&mut self, idx: DefaultNodeIdx) {
        self.enabled_layouts.remove(&idx);
        self.disabled_layouts.remove(&idx);
        self.nodes.get_mut(&self.graph.get_graph()[idx].data.user_data).unwrap().node_rect = Rect::new(0.,0.,0.,0.);
    }

    pub fn increase_node_mass(&mut self, idx: u32) {
        if let Some(node) = self.nodes.get_mut(&idx) {
            if let Some(fg_idx) = node.fg_index {
                self.graph.visit_nodes_mut(|fg_node| {
                    if fg_node.index() == fg_idx {
                        fg_node.data.mass += DEFAULT_MASS_INCREASE_AMOUNT;
                        self.animating = true;
                        if fg_node.data.mass > DEFAULT_MASS_INCREASE_AMOUNT {
                            fg_node.data.mass = fg_node.data.mass.round();
                        }
                    }
                });
            }
        }
    }

    pub fn decrease_node_mass(&mut self, idx: u32) {
        if let Some(node) = self.nodes.get_mut(&idx) {
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
                    }
                });
            }
        }
    }

    pub fn reset_node_mass(&mut self, idx: u32) {
        if let Some(node) = self.nodes.get_mut(&idx) {
            if let Some(fg_idx) = node.fg_index {
                self.graph.visit_nodes_mut(|fg_node| {
                    if fg_node.index() == fg_idx {
                        fg_node.data.mass = DEFAULT_NODE_MASS;
                        self.animating = true;
                    }
                });
            }
        }
    }

    pub fn restart_simulation(&mut self) {
        self.animating = true;
    }

    pub fn toggle_node_anchor(&mut self, idx: u32) {
        if let Some(node) = self.nodes.get_mut(&idx) {
            if self.graph.get_graph()[node.fg_index.unwrap()].data.is_anchor {
                if !self.graph.is_sole_anchor_in_component(node.fg_index.unwrap()) {
                    self.graph.get_graph_mut()[node.fg_index.unwrap()].toggle_anchor();
                    self.animating = true;
                }
            } else {
                self.graph.get_graph_mut()[node.fg_index.unwrap()].toggle_anchor();
                self.animating = true;
            }
        }
    }

    pub fn move_node(&mut self, idx: u32, vec: Vec2) {
        //Allow only non-root nodes to be moved
        if idx != 0 {
            if let Some(node) = self.nodes.get_mut(&idx) {
                if !self.graph.get_graph()[node.fg_index.unwrap()].data.is_anchor {
                    self.toggle_node_anchor(idx);
                }
            }
            if let Some(node) = self.nodes.get_mut(&idx) {
                if let Some(fg_idx) = node.fg_index {
                    self.graph.visit_nodes_mut(|fg_node| {
                        if fg_node.index() == fg_idx {
                            fg_node.data.x += vec.x;
                            fg_node.data.y += vec.y;
                        }
                    })
                }
            }
            self.animating = true;
        }
    }

    //Determine of a given Point (usually a click) intersects with a node. Return that node's index if so.
    pub fn does_point_collide(&mut self, point: Point) -> Option<u32> {
        // self.last_click_point = Some(point);
        let mut collided_index: Option<u32> = None;
        self.nodes.iter().for_each(|item| {
            let affine_scale = Affine::scale(self.scale.as_tuple().1);
            let affine_translate = Affine::translate(self.translate.as_tuple().0);
            let node = item.1;
            let node_pos = &self.get_node_pos(node.index).clone();
            let mut size = self.enabled_layouts[&node.fg_index.unwrap()].size();
            if size.width < DEFAULT_MIN_NODE_WIDTH_DATA {
                size.width = DEFAULT_MIN_NODE_WIDTH_DATA;
            }
            let mut rect = size.to_rect();
            let border = DEFAULT_BORDER_WIDTH*self.scale.as_tuple().1;
            rect = rect.inflate(border*2.0,border*2.0);
            let pos_translate = Affine::translate(
                (affine_scale * (
                    *node_pos - Vec2::new(size.width/2.0, size.height/2.0)
                ).to_point()).to_vec2()
            );
            rect = affine_scale.transform_rect_bbox(rect);
            rect = (affine_translate).transform_rect_bbox(rect);
            rect = (pos_translate).transform_rect_bbox(rect);
            if rect.contains(point) {
                collided_index = Some(node.index);
            }
        });
        collided_index
    }

    pub fn screen_point_to_canvas_point(&self, point: Point) -> Point {
        let canvas_space_pos = Affine::from(self.scale).inverse() * (self.translate.inverse() * point);
        return canvas_space_pos;
    }

    pub fn canvas_point_to_screen_point(&self, point: Point) -> Point {
        let screen_space_pos = Affine::from(self.scale) * (self.translate * point);
        return screen_space_pos
    }

    pub fn screen_point_to_label_point(&mut self, point: Point) -> (Option<u32>, Point) {
        if let Some(idx) = self.does_point_collide(point) {
            let canvas_space_pos = Affine::from(self.scale).inverse() * (self.translate.inverse() * point);
            let label_pos = self.get_node_pos(idx);
            let fg_idx = self.nodes.get(&idx).unwrap().fg_index.unwrap();
            let label_offset = self.enabled_layouts.get(&fg_idx).unwrap().size().to_vec2() / 2.;
            return (Some(idx), canvas_space_pos-label_pos+label_offset);
        } else {
            let canvas_space_pos = Affine::from(self.scale).inverse() * (self.translate.inverse() * point);
            return (None, canvas_space_pos);
        }
    }

    pub fn zoom_canvas(&mut self, factor: f64, center_point: Option<Point>) {
        let point1: Point;
        let point2: Point;
        if let Some(point) = center_point {
            if factor < 1. {
                if self.zoom_level_index != 0 {self.zoom_level_index -= 1;}
            } else if factor > 1. {
                if self.zoom_level_index < ZOOM_LEVELS.len()-1 {self.zoom_level_index += 1;}
            }
            point1 = self.screen_point_to_canvas_point(point);
            self.scale = TranslateScale::scale(ZOOM_LEVELS[self.zoom_level_index]);
            point2 = self.screen_point_to_canvas_point(point);
            let delta = self.canvas_point_to_screen_point(point2) - self.canvas_point_to_screen_point(point1);
            self.offset_x += delta.x;
            self.offset_y += delta.y;
        }
    }

    //Loop over node label generation until it fits within a set of BoxConstraints. Wraps the contents
    // once and then, if it still doesn't fit, reduce the font until it does.
    pub fn build_label_layout_for_constraints(factory: &mut PietText, text: String, bc: BoxConstraints, color: &Color) -> Result<PietTextLayout, String> {
        let mut layout: PietTextLayout;
        let mut font_size = DEFAULT_LABEL_FONT_SIZE;
        let max_width = NODE_LABEL_MAX_CONSTRAINTS.0;

        if bc.max().width < DEFAULT_MIN_NODE_WIDTH_DATA {
            return Err(format!("Cannot build label smaller than {}", DEFAULT_MIN_NODE_WIDTH_DATA));
        }

        if let Ok(layout) = factory.new_text_layout(text.clone())
        .font(FontFamily::SANS_SERIF, font_size)
        .text_color((*color).clone())
        .max_width(max_width)
        .build() {
            if bc.contains(layout.size()) {
                return Ok(layout);
            }
        }

        loop {
            if let Ok(built) = factory.new_text_layout(text.clone()) 
            .font(FontFamily::SANS_SERIF, font_size)
            .text_color(color.clone())
            .max_width(max_width)
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

    pub fn scroll_node_into_view(&mut self, idx: u32) {
        if let Some(node) = self.nodes.get(&idx) {
            if !node.node_rect.is_empty() {
                self.scroll_rect_into_view(node.node_rect);
            }
        }
    }

    pub fn set_config(&mut self, config: VMConfigVersion4) {
        self.config = config;
    }

    fn handle_action(&mut self, ctx: &mut EventCtx, payload: &ActionPayload) -> Result<(), ()> {
        if payload.action != Action::ChangeModeWithTimeoutRevert {
            // self.input_manager.clear_timeout();
        }
        match payload.action {
            Action::ChangeModeWithTimeoutRevert => {
                let current_mode = Some(self.input_manager.get_keybind_mode());
                self.input_manager.set_timeout_revert_mode(current_mode);
                self.input_manager.set_keybind_mode(payload.mode.clone().unwrap());

                match payload.mode {
                    Some(KeybindMode::SearchEntry) | Some(KeybindMode::SearchedSheet) => {
                        self.set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                    },
                    _ => {
                        self.set_render_mode(NodeRenderMode::AllEnabled);
                    }
                }
                return Ok(());
            },
            Action::ChangeMode => {
                match payload.mode {
                    Some(KeybindMode::Move) => {
                        if let Some(active_idx) = self.get_active_node_idx() {
                            if active_idx == 0 {
                                ()
                            } else {
                                self.input_manager.set_keybind_mode(payload.mode.unwrap());
                            }
                        }
                    }
                    Some(KeybindMode::SearchEntry) => {
                        self.input_manager.set_keybind_mode(payload.mode.unwrap());
                        self.set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                    },
                    Some(KeybindMode::SearchedSheet) => {
                        self.input_manager.set_keybind_mode(payload.mode.unwrap());
                        if self.get_target_list_length() == 1 {
                            ctx.submit_command(EXECUTE_ACTION.with(
                                ActionPayload {
                                    action: Action::ActivateTargetedNode,
                                    ..Default::default()
                                }
                            ));
                            self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                        } else if self.get_target_list_length() == 0 {
                            let idx = if let Some(idx) = self.get_active_node_idx() {
                                    idx
                                } else {
                                    0
                                };
                            self.input_manager.set_keybind_mode(KeybindMode::Sheet);
                            self.set_render_mode(NodeRenderMode::AllEnabled);
                            self.build_target_list_from_neighbors(idx);
                            self.cycle_target_forward();
                        }
                    },
                    Some(KeybindMode::Edit) | Some(KeybindMode::Insert) | Some(KeybindMode::Visual) => {
                        if let Some(active_node) = self.nodes.get(&self.get_active_node_idx().unwrap()) {
                            self.input_manager.text_input.text = active_node.label.clone();
                            let _ = self.input_manager.text_input.set_cursor(Some(active_node.text_cursor_index));
                        }
                        self.input_manager.set_keybind_mode(payload.mode.unwrap());
                        self.input_manager.text_input.set_keybind_mode(payload.mode.unwrap());
                        self.set_render_mode(NodeRenderMode::AllEnabled);
                    },
                    Some(KeybindMode::Sheet) => {
                        self.input_manager.set_keybind_mode(payload.mode.unwrap());
                        self.set_render_mode(NodeRenderMode::AllEnabled);
                    },
                    _ => {
                        self.input_manager.set_keybind_mode(payload.mode.unwrap());
                        self.set_render_mode(NodeRenderMode::AllEnabled);
                    }
                }
                return Ok(());
            }
            Action::NullAction => {
                return Ok(());
            },
            Action::CreateNewNode => {
                if let Some(idx) = payload.index {
                    let _ = self.add_node(idx, format!(""));
                } else if let Some(idx) = self.get_active_node_idx() {
                    let _ = self.add_node(idx, format!(""));
                }
                ctx.request_layout();
                return Ok(());
            },
            Action::InsertNewNode => {
                if let Some(to_idx) = payload.index {
                    if self.target_node_list.contains(&to_idx) {
                        if let Some(from_idx) = self.get_active_node_idx() {
                            self.insert_node(from_idx, to_idx, format!(""));
                            ctx.request_layout();
                        }
                    }
                } else if let Some(from_idx) = self.get_active_node_idx() {
                    if let Some(to_idx) = self.get_target_node_idx() {
                        self.insert_node(from_idx, to_idx, format!(""));
                        ctx.request_layout();
                    }
                }
                return Ok(());
            }
            Action::CreateNewNodeAndEdit => {
                if let Some(idx) = self.get_active_node_idx() {
                    if let Some(new_idx) = self.add_node(idx, format!("")) {
                        self.set_node_as_active(new_idx);
                        self.input_manager.set_keybind_mode(KeybindMode::Insert);
                        self.input_manager.text_input.text = self.nodes.get(&new_idx).unwrap().label.clone();
                        self.input_manager.text_input.cursor_to_start();
                    }
                }
                return Ok(());
            },
            Action::CreateNewExternalNode => {
                if let Some(_) = self.get_active_node_idx() {
                    if let Some(new_idx) = self.add_external_node(format!("New External Node")) {
                        self.set_node_as_active(new_idx);
                        if let Some(point) = payload.pos {
                            let node = self.nodes.get(&new_idx).unwrap().fg_index.unwrap();
                            self.graph.get_graph_mut()[node].data.x = self.screen_point_to_canvas_point(point).x;
                            self.graph.get_graph_mut()[node].data.y = self.screen_point_to_canvas_point(point).y;
                            ctx.request_layout();
                        } else {
                            ctx.submit_command(Command::new(
                                EXECUTE_ACTION,
                                ActionPayload {
                                    action: Action::ChangeMode,
                                    mode: Some(KeybindMode::Move),
                                    ..Default::default()
                                },
                                Target::Global
                            ));
                        }
                    }
                }
                return Ok(());
            },
            Action::EditActiveNodeSelectAll => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.input_manager.set_keybind_mode(KeybindMode::Edit);
                    self.input_manager.text_input.text = self.nodes.get(&idx).unwrap().label.clone();
                    self.input_manager.text_input.cursor_to_end();
                }
                return Ok(());
            },
            Action::EditActiveNodeInsert => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.input_manager.set_keybind_mode(KeybindMode::Insert);
                    self.input_manager.text_input.text = self.nodes.get(&idx).unwrap().label.clone();
                    self.input_manager.text_input.cursor_to_start();
                }
                return Ok(());
            },
            Action::EditActiveNodeAppend => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.input_manager.set_keybind_mode(KeybindMode::Insert);
                    self.input_manager.text_input.text = self.nodes.get(&idx).unwrap().label.clone();
                    self.input_manager.text_input.cursor_to_end();
                }
                return Ok(());
            },
            Action::CycleNodeForward => {
                if let Some(_) = self.get_active_node_idx() {
                    self.cycle_target_forward();
                    if let Some(idx) = self.get_target_node_idx() {
                        self.scroll_node_into_view(idx)
                    }
                } else {
                    self.set_node_as_active(0);
                    self.scroll_node_into_view(0);
                }
                return Ok(());
            }
            Action::CycleNodeBackward => {
                if let Some(_) = self.get_active_node_idx() {
                    self.cycle_target_backward();
                    if let Some(idx) = self.get_target_node_idx() {
                        self.scroll_node_into_view(idx)
                    }
                } else {
                    self.set_node_as_active(0);
                    self.scroll_node_into_view(0);
                }
                return Ok(());
            }
            Action::ActivateTargetedNode => {
                self.set_render_mode(NodeRenderMode::AllEnabled);
                if let Some(idx) = self.target_node_idx {
                    let node_idx = self.target_node_list[idx];
                    self.scroll_node_into_view(node_idx);
                    self.invalidate_node_layouts();
                    self.set_node_as_active(node_idx);
                    ctx.set_handled();
                }
                return Ok(());
            },
            Action::CutNodeTree => {
                let idx = payload.index.unwrap();
                if let Ok(idx) = self.cut_node_tree(idx, ctx) {
                    self.set_node_as_active(idx);
                    self.scroll_node_into_view(idx);
                }
                return Ok(());
            },
            Action::CutNode => {
                if let Some(idx) = payload.index {
                    let neighbor_count = self.graph.get_graph().neighbors(self.nodes.get(&idx).unwrap().fg_index.unwrap()).count();
                    if neighbor_count > 2 {
                        return Err(());
                    } else if neighbor_count == 2 {
                        if let Ok(idx) = self.cut_node(idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    } else {
                        if let Ok(idx) = self.cut_node_tree(idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    }
                } else if let Some(active_idx) = self.get_active_node_idx() {
                    let neighbor_count = self.graph.get_graph().neighbors(self.nodes.get(&active_idx).unwrap().fg_index.unwrap()).count();
                    if neighbor_count > 2 {
                        return Err(());
                    } else if neighbor_count == 2 {
                        if let Ok(idx) = self.cut_node(active_idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    } else {
                        if let Ok(idx) = self.cut_node_tree(active_idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    }
                }
                return Ok(());
            }
            Action::AttemptNodeDeletion => {
                if let Some(remove_idx) = payload.index {
                    let count = self.get_node_deletion_count(remove_idx);
                    if count == 0 {
                        return Ok(());
                    }
                    if count <= 1 {
                        if let Ok(idx) = self.cut_node_tree(remove_idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    } else if remove_idx == 0 {
                        
                    } else {
                        ctx.submit_command(Command::new(
                            EXECUTE_ACTION,
                            ActionPayload {
                                action: Action::CreateDialog,
                                dialog_params: Some(VMDialog::make_cut_node_prompt_dialog_params(count, remove_idx)),
                                ..Default::default()
                            },
                            Target::Global
                        ));
                    }
                } else if let Some(remove_idx) = self.get_active_node_idx() {
                    let count = self.get_node_deletion_count(remove_idx);
                    if count == 0 {
                        return Ok(());
                    }
                    if count <= 1 {
                        if let Ok(idx) = self.cut_node_tree(remove_idx, ctx) {
                            self.set_node_as_active(idx);
                            self.scroll_node_into_view(idx);
                        }
                    } else if remove_idx == 0 {
                        
                    } else {
                        ctx.submit_command(Command::new(
                            EXECUTE_ACTION,
                            ActionPayload {
                                action: Action::CreateDialog,
                                dialog_params: Some(VMDialog::make_cut_node_prompt_dialog_params(count, remove_idx)),
                                ..Default::default()
                            },
                            Target::Global
                        ));
                    }
                }
                return Ok(());
            }
            Action::CutTargetNode => {
                return Ok(());
            },
            Action::YankNode => {
                if let Some(yank_idx) = payload.index {
                    let _ = self.yank_node(yank_idx, ctx);
                } else if let Some(yank_idx) = self.get_active_node_idx() {
                    let _ = self.yank_node(yank_idx, ctx);
                }
                return Ok(());
            },
            Action::YankNodeTree => {
                if let Some(yank_idx) = payload.index {
                    let _ = self.yank_node_tree(yank_idx, ctx);
                } else if let Some(yank_idx) = self.get_active_node_idx() {
                    let _ = self.yank_node_tree(yank_idx, ctx);
                }
                return Ok(());
            },
            Action::PasteNodeTree => {
                if let Some(idx) = payload.index {
                    self.set_node_as_active(idx);
                }
                if let Some(_) = self.get_active_node_idx() {
                    ctx.submit_command(Command::new(GET_REGISTER,
                        ("0".to_string(), false, None),
                        Target::Global
                    ));
                    return Ok(());
                } else {
                    return Ok(());
                }
            }
            Action::PasteNodeTreeExternal => {
                if let Some(point) = payload.pos {
                    ctx.submit_command(Command::new(GET_REGISTER,
                    ("0".to_string(), true, Some(point)),
                    Target::Global));
                } else {
                    ctx.submit_command(Command::new(GET_REGISTER,
                    ("0".to_string(), true, None),
                    Target::Global));
                }
                return Ok(());
            }
            Action::IncreaseNodeMass => {
                if let Some(idx) = payload.index {
                    self.increase_node_mass(idx);
                } else if let Some(idx) = self.get_active_node_idx() {
                    self.increase_node_mass(idx);
                }
                return Ok(());
            }
            Action::DecreaseNodeMass => {
                if let Some(idx) = payload.index {
                    self.decrease_node_mass(idx);
                } else if let Some(idx) = self.get_active_node_idx() {
                    self.decrease_node_mass(idx);
                }
                return Ok(());
            }
            Action::ResetNodeMass => {
                if let Some(idx) = payload.index {
                    self.reset_node_mass(idx);
                } else if let Some(idx) = self.get_active_node_idx() {
                    self.reset_node_mass(idx);
                }
                return Ok(());
            }
            Action::ToggleNodeAnchor => {
                if let Some(idx) = payload.index {
                    self.toggle_node_anchor(idx);
                } else if let Some(idx) = self.get_active_node_idx() {
                    self.toggle_node_anchor(idx);
                }
                return Ok(());
            }
            Action::MoveActiveNodeDown => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.move_node(idx, Vec2::new(0., payload.float.expect("Expected a float value for node movement.")))
                }
                return Ok(());
            }
            Action::MoveActiveNodeUp => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.move_node(idx, Vec2::new(0., -1.*payload.float.expect("Expected a float value for node movement.")))
                }
                return Ok(());
            }
            Action::MoveActiveNodeLeft => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.move_node(idx, Vec2::new(-1.*payload.float.expect("Expected a float value for node movement."), 0.))
                }
                return Ok(());
            }
            Action::MoveActiveNodeRight => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.move_node(idx, Vec2::new(payload.float.expect("Expected a float value for node movement."), 0.))
                }
                return Ok(());
            }
            Action::MarkActiveNode => {
                if let Some(active_idx) = self.get_active_node_idx() {
                    //Check that the chosen node isn't a root. Do nothing and return immediately if so.
                    if self.is_node_root(active_idx) {return Ok(());}
                    //Check that a node doesn't already have this mark. Clear if that's the case.
                    if let Some(holder) = self.get_node_by_mark(payload.string.clone().unwrap()) {
                        self.set_node_mark(holder, " ".to_string());
                        // self.nodes.get_mut(&holder).unwrap().set_mark(" ".to_string());
                    }
                    // self.nodes.get_mut(&active_idx).unwrap().set_mark(payload.string.clone().unwrap());
                    self.set_node_mark(active_idx, payload.string.clone().unwrap());
                }
                return Ok(());
            },
            Action::JumpToMarkedNode => {
                if let Some(marked_idx) = self.get_node_by_mark(payload.string.clone().unwrap()) {
                    self.set_node_as_active(marked_idx);
                    self.scroll_node_into_view(marked_idx);
                }
                return Ok(());
            },
            Action::TargetNode => todo!(),
            Action::CenterNode => {
                let node = self.nodes.get(&payload.index.unwrap()).expect("Tried to center a non-existent node.");
                let node_pos = self.get_node_pos(node.index) * self.scale.as_tuple().1;
                self.offset_x = node_pos.x;
                self.offset_y = node_pos.y;
                return Ok(());
            }
            Action::CenterActiveNode => {
                if let Some(active_idx) = self.get_active_node_idx() {
                    let node = self.nodes.get(&active_idx).expect("Tried to get non-existent active node.");
                    let node_pos = self.get_node_pos(node.index) * self.scale.as_tuple().1;
                    self.offset_x = -1. * node_pos.x;
                    self.offset_y = -1. * node_pos.y;
                }
                return Ok(());
            }
            Action::SearchNodes => {
                if let Some(string) = payload.string.clone() {
                    if let Ok(_) = self.build_target_list_from_string(string) {

                    }
                    // self.set_render_mode(NodeRenderMode::OnlyTargetsEnabled);
                }
                return Ok(());
            },
            Action::PanUp => {
                self.offset_y += payload.float.unwrap();
                return Ok(());
            }
            Action::PanDown => {
                self.offset_y -= payload.float.unwrap();
                return Ok(());
            }
            Action::PanLeft => {
                self.offset_x += payload.float.unwrap();
                return Ok(());
            }
            Action::PanRight => {
                self.offset_x -= payload.float.unwrap();
                return Ok(());
            }
            Action::ZoomOut => {
                let center_point = Point::new(self.canvas_rect.unwrap().width()/2., self.canvas_rect.unwrap().height()/2.);
                self.zoom_canvas(payload.float.unwrap(), Some(center_point));
                return Ok(());
            }
            Action::ZoomIn => {
                let center_point = Point::new(self.canvas_rect.unwrap().width()/2., self.canvas_rect.unwrap().height()/2.);
                self.zoom_canvas(payload.float.unwrap(), Some(center_point));
                return Ok(());
            },
            Action::AcceptNodeText => {
                if let Some(idx) = self.get_active_node_idx() {
                    let node = self.nodes.get_mut(&idx).unwrap();
                    let node_fg = node.fg_index.unwrap();
                    node.label = self.input_manager.text_input.text.clone();
                    node.text_cursor_index = self.input_manager.text_input.get_cursor_index();
                    self.invalidate_node_layout(node_fg);
                    self.animating = true;
                }
                return Ok(());
            },
            Action::ExecuteTextAction |
            Action::InsertCharacterUnconfirmed |
            Action::ConfirmInserts |
            Action::RollBackInserts |
            Action::InsertCharacter => {
                let ret = self.input_manager.text_input.handle_action(ctx, payload);
                if let Some(active_idx) = self.get_active_node_idx() {
                    self.nodes.get_mut(&active_idx).unwrap().label = self.input_manager.text_input.text.clone();
                    self.nodes.get_mut(&active_idx).unwrap().text_cursor_index = self.input_manager.text_input.get_cursor_index();
                    self.invalidate_node_layouts();
                    self.animating = true;
                }
                if let Some(mode) = ret {
                    self.input_manager.set_keybind_mode(mode);
                }
                return Ok(());
            },
            _ => {
                return Ok(());
            }
        }
    }
}

impl Widget<()> for VimMapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        if self.animating { ctx.request_anim_frame(); }
        //If the node editor is visible, pass events to it. Both events and paints must be withheld
        // for the widget to be truly hidden and uninteractable. 
        match event {
            Event::AnimFrame(_interval) => {
                // if self.is_hot && self.animating {
                if self.animating {
                    self.largest_node_movement = Some(self.graph.update(DEFAULT_UPDATE_DELTA));
                    if self.largest_node_movement < Some(ANIMATION_MOVEMENT_THRESHOLD) && self.animation_timer_token == None {
                        // self.animating = false;
                        self.animation_timer_token = Some(ctx.request_timer(DEFAULT_ANIMATION_TIMEOUT));
                    }
                }
                ctx.request_update();
                ctx.request_layout();
                ctx.request_paint();
            }
            Event::MouseDown(mouse_event) if mouse_event.button.is_left() => {
                if self.input_manager.get_keybind_mode() == KeybindMode::Move {
                    ctx.submit_command(Command::new(EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        },
                        Target::Global
                    ))    
                } else if mouse_event.count == 1 {
                    match self.input_manager.get_keybind_mode() {
                        KeybindMode::Edit |
                        KeybindMode::Insert |
                        KeybindMode::Visual => {
                            if let Some(idx) = self.does_point_collide(mouse_event.pos) {
                                if let Some(active_idx) = self.get_active_node_idx() {
                                    if let Some(target_idx) = self.get_target_node_idx() {
                                        if idx != active_idx {
                                            let _ = self.handle_action(ctx, &ActionPayload {
                                                action: Action::ChangeMode,
                                                mode: Some(KeybindMode::Sheet),
                                                ..Default::default()
                                            });
                                            self.last_mouse_down_data = Some((Some(idx), mouse_event.pos, (self.offset_x, self.offset_y)));
                                            if target_idx == idx {
                                                self.set_node_as_active(idx);
                                                let _ = self.target_node_if_listed(active_idx);
                                            } else if let Ok(_) = self.target_node_if_listed(idx) {

                                            } else {
                                                self.set_node_as_active(idx);
                                            }
                                        } else {
                                            //handle text input click location here
                                            if let (Some(hit_idx), point) = self.screen_point_to_label_point(mouse_event.pos) {
                                                let index = self.input_manager.text_input.get_index_at_point(point);
                                                if let Ok(index) = index {
                                                    if hit_idx == idx {
                                                        let _ = self.input_manager.text_input.set_cursor(Some(index));
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        if idx != active_idx {
                                            let _ = self.handle_action(ctx, &ActionPayload {
                                                action: Action::ChangeMode,
                                                mode: Some(KeybindMode::Sheet),
                                                ..Default::default()
                                            });
                                            self.last_mouse_down_data = Some((Some(idx), mouse_event.pos, (self.offset_x, self.offset_y)));
                                            if let Ok(_) = self.target_node_if_listed(idx) {

                                            } else {
                                                self.set_node_as_active(idx);
                                            }
                                        } else {
                                            //handle text input click location here
                                            if let (Some(hit_idx), point) = self.screen_point_to_label_point(mouse_event.pos) {
                                                let index = self.input_manager.text_input.get_index_at_point(point);
                                                if let Ok(index) = index {
                                                    if hit_idx == idx {
                                                        let _ = self.input_manager.text_input.set_cursor(Some(index));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    let _ = self.handle_action(ctx, &ActionPayload {
                                        action: Action::ChangeMode,
                                        mode: Some(KeybindMode::Sheet),
                                        ..Default::default()
                                    });
                                    self.last_mouse_down_data = Some((Some(idx), mouse_event.pos, (self.offset_x, self.offset_y)));
                                    self.set_node_as_active(idx);
                                }
                            } else {
                                let _ = self.handle_action(ctx, &ActionPayload {
                                    action: Action::ChangeMode,
                                    mode: Some(KeybindMode::Sheet),
                                    ..Default::default()
                                });
                                self.last_mouse_down_data = Some((None, mouse_event.pos, (self.offset_x, self.offset_y)));
                            }

                        }
                        _ => {
                            if let Some(idx) = self.does_point_collide(mouse_event.pos) {
                                self.last_mouse_down_data = Some((Some(idx), mouse_event.pos, (self.offset_x, self.offset_y)));
                                if let Some(active_idx) = self.get_active_node_idx() {
                                    if let Some(target_idx) = self.get_target_node_idx() {
                                        if target_idx == idx {
                                            self.set_node_as_active(idx);
                                            let _ = self.target_node_if_listed(active_idx);
                                        } else if let Ok(_) = self.target_node_if_listed(idx) {

                                        } else {
                                            self.set_node_as_active(idx);
                                        }
                                    } else {
                                        if let Ok(_) = self.target_node_if_listed(idx) {
                                            let _ = self.target_node_if_listed(active_idx);
                                         } else {
                                            self.set_node_as_active(idx);
                                        }
                                    }
                                } else {
                                    self.set_node_as_active(idx);
                                }
                            } else {
                                self.last_mouse_down_data = Some((None, mouse_event.pos, (self.offset_x, self.offset_y)));
                            }
                        }
                    }
                } else if mouse_event.count == 2 {
                    self.last_mouse_down_data = None;
                    if let Some(idx) = self.does_point_collide(mouse_event.pos) {
                        self.set_node_as_active(idx);
                        let _ = self.handle_action(ctx, &ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Insert),
                            ..Default::default()
                        });
                    }
                }
            }
            Event::MouseDown(mouse_event) if mouse_event.button.is_right() => {
                if self.input_manager.get_keybind_mode() == KeybindMode::Move {
                    ctx.submit_command(Command::new(EXECUTE_ACTION,
                        ActionPayload {
                            action: Action::ChangeMode,
                            mode: Some(KeybindMode::Sheet),
                            ..Default::default()
                        },
                        Target::Global
                    ))    
                } else if let Some(idx) = self.does_point_collide(mouse_event.pos) {
                    self.last_menu_pos = mouse_event.pos;
                    ctx.show_context_menu(self.build_menu_for_node(ctx, idx), mouse_event.pos);
                } else {
                    // global ctx menu here
                    self.last_menu_pos = mouse_event.pos;
                    ctx.show_context_menu(self.build_menu_for_sheet(ctx, Some(mouse_event.pos)), mouse_event.pos);
                }
            }
            Event::MouseUp(_) => {
                self.last_mouse_down_data = None;
            }
            Event::MouseMove(mouse_event) => {
                if self.input_manager.get_keybind_mode() == KeybindMode::Move && self.last_mouse_down_data == None {
                    self.last_mouse_down_data = Some((self.get_active_node_idx(), mouse_event.pos, (self.offset_x, self.offset_y)));
                }
                if self.input_manager.get_keybind_mode() == KeybindMode::Move {
                    if let None = self.last_mouse_down_data {
                        self.last_mouse_down_data = Some((self.get_active_node_idx(), mouse_event.pos, (self.offset_x, self.offset_y)));
                    } else if let Some((None, _, _)) = self.last_mouse_down_data {
                        self.last_mouse_down_data = Some((self.get_active_node_idx(), mouse_event.pos, (self.offset_x, self.offset_y)));
                    }
                    if let Some((Some(idx), pos, _last_translate)) = self.last_mouse_down_data {
                        if idx != 0 {
                            let delta = pos - mouse_event.pos;
                            if delta.hypot() > DEFAULT_NODE_DRAG_THRESHOLD {
                                let canvas_space_pos = Affine::from(self.scale).inverse() * (self.translate.inverse() * mouse_event.pos);
                                let fg_idx = self.nodes.get(&idx).unwrap().fg_index.unwrap();
                                self.graph.get_graph_mut()[fg_idx].data.is_anchor = true;
                                self.graph.get_graph_mut()[fg_idx].data.x = canvas_space_pos.x;
                                self.graph.get_graph_mut()[fg_idx].data.y = canvas_space_pos.y;
                                self.animating = true;
                            }
                        }
                    }
                } else if let Some((Some(idx), pos, _last_translate)) = self.last_mouse_down_data {
                    if idx != 0 {
                        let delta = pos - mouse_event.pos;
                        if delta.hypot() > DEFAULT_NODE_DRAG_THRESHOLD {
                            let canvas_space_pos = Affine::from(self.scale).inverse() * (self.translate.inverse() * mouse_event.pos);
                            let fg_idx = self.nodes.get(&idx).unwrap().fg_index.unwrap();
                            self.graph.get_graph_mut()[fg_idx].data.is_anchor = true;
                            self.graph.get_graph_mut()[fg_idx].data.x = canvas_space_pos.x;
                            self.graph.get_graph_mut()[fg_idx].data.y = canvas_space_pos.y;
                            self.animating = true;
                        }
                    }
                } else if let Some((None, pos, last_translate)) = self.last_mouse_down_data {
                    let delta = pos - mouse_event.pos;
                    self.offset_x = last_translate.0 - delta.x;
                    self.offset_y = last_translate.1 - delta.y;
                }
            }
            Event::Wheel(mouse_event) => {
                if mouse_event.mods.shift() {
                    self.offset_x -= mouse_event.wheel_delta.to_point().x;
                } else if mouse_event.mods.ctrl() {
                    if mouse_event.wheel_delta.to_point().y < 0.0 {
                        self.zoom_canvas(0.75, Some(mouse_event.pos));
                    } else {
                        self.zoom_canvas(1.25, Some(mouse_event.pos));
                    }
                } else {
                    self.offset_y -= mouse_event.wheel_delta.to_point().y;
                    self.offset_x -= mouse_event.wheel_delta.to_point().x;
                }
                ctx.request_anim_frame();
            }
            Event::Timer(event) => {
                if Some(*event) == self.animation_timer_token {
                    ctx.set_handled();
                    if let Some(delta) = self.largest_node_movement {
                        if delta < ANIMATION_MOVEMENT_THRESHOLD {
                            self.animating = false;
                            self.animation_timer_token = None;
                        } else {
                            self.animation_timer_token = Some(ctx.request_timer(DEFAULT_ANIMATION_TIMEOUT));
                        }
                    }
                }
                ctx.request_anim_frame();
            }
            Event::Command(note) if note.is(REFRESH) => {
                self.invalidate_node_layouts();
                ctx.request_update();
                ctx.request_layout();
                ctx.request_anim_frame();
                ctx.set_handled();
            }
            Event::Command(command) if command.is(OFFER_REGISTER) && !ctx.is_handled() => {
                if let Some(active_idx) = self.get_active_node_idx() {
                    let (register, graph_clip, is_external, external_point) = command.get_unchecked(OFFER_REGISTER).clone();
                    if !is_external {
                        graph_clip.append_node_clip(self, Some(active_idx), register, None);
                    } else {
                        graph_clip.append_node_clip(self, None, register, external_point);
                    }
                }
            }
            Event::Command(command) if command.is(EXECUTE_ACTION) && !ctx.is_handled() => {
                let payload = command.get::<ActionPayload>(EXECUTE_ACTION).unwrap();
                if let Ok(_) = self.handle_action(ctx, payload) {
                    ctx.set_handled();
                }
                ctx.request_anim_frame();
            }
            _ => {
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &(), _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                if let Some(idx) = self.get_active_node_idx() {
                    self.build_target_list_from_neighbors(idx);
                    self.cycle_target_forward();
                }
                //Register children with druid
                //Kick off animation and calculation
                ctx.request_layout();
                ctx.request_anim_frame();
                ctx.children_changed();
            },
            LifeCycle::HotChanged(is_hot) => {
                //Cache is_hot values
                self.is_hot = *is_hot;
                self.last_mouse_down_data = None;
                ctx.request_anim_frame();
            },
            _ => {
            }
        }
    }
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &(), _env: &Env) -> Size {
        if let Some(rect) = self.canvas_rect {
            let vec = rect.size();
            self.translate = TranslateScale::new((vec.to_vec2()/2.0)+Vec2::new(self.offset_x, self.offset_y), 1.0);
        }

        self.graph.visit_nodes_mut(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data).unwrap();
            if let None = self.enabled_layouts.get(&fg_node.index()) {
                if let Ok(layout) = VimMapper::build_label_layout_for_constraints(
                    ctx.text(), node.label.clone(), BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(NODE_LABEL_MAX_CONSTRAINTS.0, NODE_LABEL_MAX_CONSTRAINTS.1)
                    ),
                    &self.config.get_color(VMColor::LabelTextColor).ok().expect("Couldn't find label text color in config."),
                ) {
                    self.enabled_layouts.insert(fg_node.index(), layout.clone());
                    if !self.input_manager.get_keybind_mode().contains(KeybindMode::Insert) {
                        if layout.size().width < DEFAULT_MIN_NODE_WIDTH_DATA {
                            fg_node.data.repel_distance = DEFAULT_MIN_NODE_WIDTH_DATA;
                        } else {
                            fg_node.data.repel_distance = layout.size().width;
                        }
                    }
                } 
            }
            if let None = self.disabled_layouts.get(&fg_node.index()) {
                if let Ok(layout) = VimMapper::build_label_layout_for_constraints(
                    ctx.text(), node.label.clone(), BoxConstraints::new(
                        Size::new(0., 0.),
                        Size::new(NODE_LABEL_MAX_CONSTRAINTS.0, NODE_LABEL_MAX_CONSTRAINTS.1)
                    ),
                        &self.config.get_color(VMColor::DisabledLabelTextColor).ok().expect("Couldn't find disabled label text color in config."),
                ) {
                    self.disabled_layouts.insert(fg_node.index(), layout.clone());
                } 
            }
        });

        self.input_manager.text_input.layout(ctx, &self.config);

        return bc.max();
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &(), _env: &Env) {
        let vec = ctx.size();
        self.translate = TranslateScale::new((vec.to_vec2()/2.)+Vec2::new(self.offset_x, self.offset_y), 1.0);
        let ctx_size = ctx.size();
        let ctx_rect = ctx_size.to_rect();
        self.canvas_rect = Some(ctx_rect.clone());
        //Fill the canvas with background
        ctx.fill(ctx_rect, &self.config.get_color(VMColor::SheetBackgroundColor).ok().expect("sheet background color not found"));


        //Draw edges
        self.graph.visit_edges(|node1, node2, _edge| {
            let p0 = Point::new(node1.x() as f64, node1.y() as f64);
            let p1 = Point::new(node2.x() as f64, node2.y() as f64);
            let path = Line::new(p0, p1);
            ctx.with_save(|ctx| {
                ctx.transform(Affine::from(self.translate));
                ctx.transform(Affine::from(self.scale));
                ctx.stroke(path, &self.config.get_color(VMColor::EdgeColor).ok().expect("edge color not found in config"), DEFAULT_EDGE_WIDTH);
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
        let target_node: Option<u32> = self.get_target_node_idx();

        let active_node = self.get_active_node_idx();

        //Draw nodes
        self.graph.visit_nodes(|fg_node| {
            let node = self.nodes.get_mut(&fg_node.data.user_data)
            .expect("Expected non-option node in paint loop.");
            let node_pos = Vec2::new(self.graph.get_graph()[node.fg_index.unwrap()].x(), self.graph.get_graph()[node.fg_index.unwrap()].y());
            let mut enabled = true;
            if self.node_render_mode == NodeRenderMode::OnlyTargetsEnabled {
                enabled = false;
                for idx in &self.target_node_list {
                    if node.index == *idx {
                        enabled = true;
                    }
                }
            }

            match node.index {
                i if Some(i) != active_node && Some(i) != target_node => {
                    node.paint_node(
                        ctx, 
                        0,
                        &self.graph,
                        enabled,
                        if enabled {&self.enabled_layouts[&node.fg_index.unwrap()]} else {&self.disabled_layouts[&node.fg_index.unwrap()]},
                        &self.config, 
                        target_node, 
                        node_pos,
                        &self.translate, 
                        &self.scale, 
                        self.debug_data); 
                },
                _ => ()
            }
        });

        if let Some(target_idx) = target_node {
            let mut enabled = true;

            if self.get_render_mode() == NodeRenderMode::OnlyTargetsEnabled {
                enabled = if let Some(_) = self.target_node_list.iter().find(|idx| {
                    if **idx == target_idx {
                        return true;
                    } else {
                        return false;
                    }
                }) {
                    true
                } else {
                    false
                }
            };
            let target_node_pos = self.get_node_pos(target_idx);
            let node = self.nodes.get_mut(&target_idx).unwrap();
            node.paint_node(
                        ctx, 
                        0,
                        &self.graph,
                        enabled,
                        if enabled {&self.enabled_layouts[&node.fg_index.unwrap()]} else {&self.disabled_layouts[&node.fg_index.unwrap()]},
                        &self.config, 
                        target_node, 
                        target_node_pos,
                        &self.translate, 
                        &self.scale, 
                        self.debug_data); 
        }

        if let Some(active_idx) = active_node {
            let mut enabled = true;

            if self.get_render_mode() == NodeRenderMode::OnlyTargetsEnabled {
                enabled = if let Some(_) = self.target_node_list.iter().find(|idx| {
                    if **idx == active_idx {
                        return true;
                    } else {
                        return false;
                    }
                }) {
                    true
                } else {
                    false
                }
            };

            let active_node_pos = self.get_node_pos(active_idx);
            let node =self.nodes.get_mut(&active_idx).unwrap();
            node.paint_node(
                        ctx, 
                        0,
                        &self.graph,
                        enabled,
                        if self.input_manager.get_keybind_mode() != KeybindMode::Insert && self.input_manager.get_keybind_mode() != KeybindMode::Edit {
                            if enabled {&self.enabled_layouts[&node.fg_index.unwrap()]} else {&self.disabled_layouts[&node.fg_index.unwrap()]}
                        } else {
                            &self.input_manager.text_input.text_layout.as_ref().unwrap()
                        },
                        &self.config, 
                        target_node, 
                        active_node_pos,
                        &self.translate, 
                        &self.scale, 
                        self.debug_data); 
            

            //Render input label and cursor boxes if necessary
            if self.input_manager.get_keybind_mode() == KeybindMode::Insert || 
                self.input_manager.get_keybind_mode() == KeybindMode::Edit || 
                self.input_manager.get_keybind_mode() == KeybindMode::Visual {
                ctx.with_save(|ctx| {
                    let mut label_size = self.input_manager.text_input.text_layout.as_ref().unwrap().size();
                    if label_size.width < DEFAULT_MIN_NODE_WIDTH_DATA {
                        label_size.width = DEFAULT_MIN_NODE_WIDTH_DATA;
                    }
                    ctx.transform(Affine::from(self.translate));
                    ctx.transform(Affine::from(self.scale));
                    ctx.transform(Affine::translate(-1.0*(label_size.to_vec2())/2.0));
                    ctx.transform(Affine::translate(active_node_pos));
                    ctx.fill(label_size.to_rect(), &self.config.get_color(VMColor::NodeBackgroundColor).unwrap());
                    self.input_manager.text_input.paint(ctx, &self.config, self.debug_data);
                });
            }
        }


        //Paint debug dump
        if self.debug_data {
            let debug_text = format!(
                "Animating: {}\nAnimation timer: {:?}\nIs hot:{}",
                self.animating,
                self.animation_timer_token,
                self.is_hot,
            );
            let debug_layout = ctx.text().new_text_layout(debug_text)
            .font(FontFamily::SANS_SERIF, 16.)
            .text_color(self.config.get_color(VMColor::ComposeIndicatorTextColor).expect("Couldn't get indicator color"))
            .default_attribute(FontWeight::BOLD)
            .build()
            .unwrap();

            ctx.draw_text(&debug_layout, Point::new(ctx_rect.x1-debug_layout.size().width-40., ctx_rect.y1-debug_layout.size().height-40.));
        }
    }
}

