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

use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, Path};

use druid::{Vec2, Data, WidgetPod};
use druid::kurbo::TranslateScale;
use vm_force_graph_rs::{ForceGraph, DefaultNodeIdx, NodeData, EdgeData};
use serde::{Serialize, Deserialize};

use crate::{constants::*, VMTab};

use crate::vimmapper::NodeRenderMode;
use crate::vmnode::{VMNode, VMNodeEditor};
use crate::{vmconfig::VMConfigVersion4, vimmapper::VimMapper};


//A boiled-down struct to hold the essential data to serialize and deserialize a graph sheet. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Serialize, Deserialize)]
pub struct VMSaveVersion4 {
    file_version: String, 
    graph: ForceGraph<u32, u32>,
    nodes: HashMap<u32, BareNodeVersion4>,
    root_nodes: HashMap<usize, DefaultNodeIdx>,
    // edges: HashMap<u32, BareEdgeVersion4>,
    node_idx_count: u32,
    // edge_idx_count: u32,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

#[derive(Serialize, Deserialize)]
pub struct VMSaveVersion5 {
    file_version: String,
    tabs: Vec<VMTabSave>,
    active_tab: usize,
}

#[derive(Serialize, Deserialize)]
pub struct VMTabSave {
    tab_name: String,
    graph: ForceGraph<u32, u32>,
    nodes: HashMap<u32, BareNodeVersion4>,
    root_nodes: HashMap<usize, DefaultNodeIdx>,
    node_idx_count: u32,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

impl From<VMSaveVersion4> for VMSaveVersion5 {
    fn from(save: VMSaveVersion4) -> Self {
        VMSaveVersion5 {
            file_version: String::from(CURRENT_SAVE_FILE_VERSION),
            tabs: vec![
                VMTabSave { 
                    tab_name: String::from("Tab 1"),
                    graph: save.graph,
                    nodes: save.nodes, 
                    root_nodes: save.root_nodes, 
                    node_idx_count: save.node_idx_count, 
                    translate: save.translate, 
                    scale: save.scale, 
                    offset_x: save.offset_x, 
                    offset_y: save.offset_y 
                }
            ],
            active_tab: 0,
        }
    }
}

impl From<VMSaveNoVersion> for VMSaveVersion5 {
    fn from(save: VMSaveNoVersion) -> Self {
        VMSaveVersion5::from(VMSaveVersion4::from(save))
    }
}

#[derive(Data, PartialEq, Clone, Debug)]
pub enum VMSaveState {
    NoSheetOpened,
    NoSave,
    UnsavedChanges,
    SaveAsInProgress,
    SaveAsInProgressThenQuit,
    SaveAsInProgressThenNew,
    SaveAsInProgressThenOpen,
    Saved,
    DiscardChanges,
}

#[derive(Serialize, Deserialize)]
pub struct VMSaveNoVersion {
    nodes: HashMap<u32, BareNodeVersion4>,
    edges: HashMap<u32, BareEdgeVersion4>,
    node_idx_count: u32,
    edge_idx_count: u32,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

impl From<VMSaveNoVersion> for VMSaveVersion4 {
    fn from(save: VMSaveNoVersion) -> Self {
        let mut graph: ForceGraph<u32, u32> = ForceGraph::new(DEFAULT_SIMULATION_PARAMETERS);
        let mut nodes: HashMap<u32, VMNode> = HashMap::with_capacity(50);
        for (_k ,v) in &save.nodes {
            let fg_index: Option<DefaultNodeIdx>;
            if v.index == 0 {
                fg_index = Some(graph.add_node(NodeData {
                    is_anchor: true,
                    x: v.pos.0,
                    y: v.pos.1,
                    mass: v.mass,
                    user_data: {
                        0
                    },
                    ..Default::default()
                }));
            } else {
                fg_index = Some(graph.add_node(NodeData {
                    is_anchor: v.anchored,
                    x: v.pos.0,
                    y: v.pos.1,
                    mass: v.mass,
                    user_data: {
                        v.index
                    },
                    ..Default::default()
                }));
            }
            nodes.insert(v.index, VMNode {
                label: v.label.clone(), 
                index: v.index, 
                fg_index: fg_index, 
                mark: v.mark.clone(),
                ..Default::default()
            });
        }
        for (_k,v) in &save.edges {
            graph.add_edge(
                nodes.get(&v.from).unwrap().fg_index.unwrap(), 
                nodes.get(&v.to).unwrap().fg_index.unwrap(), 
                EdgeData { user_data: v.index });
        }
        tracing::debug!("coercing VMSaveNoVerion to VMSaveVersion4");
        let mut new_nodes = save.nodes.clone();
        new_nodes.get_mut(&0).unwrap().mark = Some("0".to_string());
        let mut root_nodes: HashMap<usize, DefaultNodeIdx> = HashMap::new();
        root_nodes.insert(graph.get_node_component(nodes.get(&0).unwrap().fg_index.unwrap()), nodes.get(&0).unwrap().fg_index.unwrap());
        let mut components = graph.get_components();
        if components.len() > root_nodes.len() {
            tracing::debug!("VMSaveNoVersion has extra root nodes!");
            for i in 1..components.len() {
                components[i].sort();
                root_nodes.insert(graph.get_node_component(components[i][0]), components[i][0]);
                new_nodes.get_mut(&graph.get_graph()[components[i][0]].data.user_data).unwrap().mark = Some(i.to_string());
            }
        }
        VMSaveVersion4 {
            file_version: CURRENT_SAVE_FILE_VERSION.to_string(),
            graph,
            nodes: new_nodes,
            node_idx_count: save.node_idx_count,
            translate: save.translate,
            scale: save.scale,
            offset_x: save.offset_x,
            offset_y: save.offset_y,
            root_nodes,
        }
    }
}

pub struct VMSaveSerde;

impl VMSaveSerde {
    //Instantiates a new VimMapper struct from a deserialized VMSave. The ForceGraph is created from scratch
    // and no fg_index values are guaranteed to persist from session to session.
    pub(crate) fn from_save(save: VMSaveVersion5, config: VMConfigVersion4) -> (Vec<VMTab>, usize) {
        let mut vms: Vec<VMTab> = vec![];
        for tab in save.tabs {
            let graph = tab.graph;
            let mut nodes: HashMap<u32, VMNode> = HashMap::with_capacity(50);
            for (_k ,v) in tab.nodes {
                let mut fg_index: DefaultNodeIdx = DefaultNodeIdx::default();
                graph.visit_nodes(|n| {
                    if n.data.user_data == v.index {
                        fg_index = n.index();
                    }
                });
                nodes.insert(v.index, VMNode {
                    label: v.label.clone(), 
                    index: v.index, 
                    fg_index: Some(fg_index), 
                    mark: v.mark,
                    ..Default::default()
                });
            }
            let mut vm = VimMapper {
                graph,
                animating: true,
                nodes,
                node_idx_count: tab.node_idx_count,
                translate: TranslateScale::new(
                    Vec2::new(
                        tab.translate.0, 
                        tab.translate.1),
                    0.),
                scale: TranslateScale::new(
                    Vec2::new(
                        0., 
                        0.),
                    tab.scale),
                offset_x: tab.offset_x,
                offset_y: tab.offset_y,
                target_node_list: vec![],
                target_node_idx: None,
                node_editor: VMNodeEditor::new(),
                is_hot: true,
                config: config.clone(),
                node_render_mode: NodeRenderMode::AllEnabled,
                root_nodes: tab.root_nodes,
                ..Default::default()
            };
            vm.set_node_as_active(0);
            vms.push(VMTab {vm: WidgetPod::new(vm), tab_name: tab.tab_name});
        }
        (vms, save.active_tab)
    }

    //Instantiates a serializable VMSave from the VimMapper struct. All ForceGraph data is discarded and
    // must be recreated when the VMSave is deserialized and instantiated into a VimMapper struct
    pub(crate) fn to_save(vms: &Vec<VMTab>, active_tab: usize) -> VMSaveVersion5 {
        let mut tabs: Vec<VMTabSave> = vec![];
        for tab in vms {
            let vm = tab.vm.widget();
            let mut nodes: HashMap<u32, BareNodeVersion4> = HashMap::with_capacity(50);
            vm.get_nodes().iter().for_each(|(index, node)| {
                let pos = vm.get_node_pos(*index);
                nodes.insert(*index, BareNodeVersion4 {
                    label: node.label.clone(),
                    index: node.index,
                    pos: (pos.x, pos.y),
                    is_active: false,
                    targeted_internal_edge_idx: None,
                    mark: node.mark.clone(),
                    mass: vm.graph.get_graph()[node.fg_index.unwrap()].data.mass,
                    anchored: vm.graph.get_graph()[node.fg_index.unwrap()].data.is_anchor
                });
            });
            let save = VMTabSave {
                tab_name: tab.tab_name.clone(),
                graph: vm.graph.clone(),
                nodes: nodes,
                node_idx_count: vm.get_node_idx_count(),
                translate: (vm.get_translate().as_tuple().0.x, vm.get_translate().as_tuple().0.y),
                scale: vm.get_scale().as_tuple().1,
                offset_x: vm.get_offset_x(),
                offset_y: vm.get_offset_y(),
                root_nodes: vm.root_nodes.clone(),
            };
            tabs.push(save)
        }
        VMSaveVersion5 {
            file_version: String::from(CURRENT_SAVE_FILE_VERSION),
            tabs,
            active_tab
        }
    }

    pub(crate) fn load(path: String) -> Result<(VMSaveVersion5, PathBuf), String> {
        if let Ok(string) = fs::read_to_string(path.clone()) {
            if let Ok(path) = Path::new(&path.clone()).canonicalize() {
                if let Ok(save) = serde_json::from_str::<VMSaveVersion5>(string.as_str()) {
                    return Ok((save, path));
                } else if let Ok(save) = serde_json::from_str::<VMSaveVersion4>(string.as_str()) {
                    tracing::debug!("Converting from VMSaveVersion4");
                    return Ok((VMSaveVersion5::from(save), path))
                } else if let Ok(save) = serde_json::from_str::<VMSaveNoVersion>(string.as_str()) {
                    tracing::debug!("Converting from VMSaveNoVersion");
                    return Ok((VMSaveVersion5::from(save), path));
                } else {
                    return Err(String::from("Could not serialize from save."));
                }
            } else {
                Err("Not a valid path.".to_string())
            }
        } else {
        Err("Couldn't load file.".to_string())
        }
    }

    pub(crate) fn save(save: &VMSaveVersion5, path: PathBuf) -> Result<String, String> {
        #[cfg(debug_assertions)]
        {
            println!("Saving file to {}", path.display());
        }
        if let Ok(string) = serde_json::to_string::<VMSaveVersion5>(save) {
            if let Ok(_) = fs::write(path, string) {
                Ok("File saved".to_string())
            } else {
                Err("Could not save to file.".to_string())
            }
        } else {
            Err("Could not serialize map".to_string())
        }
    }
}

//A boiled-down struct to hold the essential data to serialize and deserialize a node. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Clone, Serialize, Deserialize)]
pub struct BareNodeVersion4 {
    label: String,
    // edges: Vec<u32>,
    index: u32,
    pos: (f64, f64),
    is_active: bool,
    mark: Option<String>,
    targeted_internal_edge_idx: Option<usize>,
    mass: f64,
    anchored: bool,
}

impl Default for BareNodeVersion4 {
    fn default() -> Self {
        BareNodeVersion4 { 
            label: DEFAULT_NEW_NODE_LABEL.to_string(),
            // edges: vec![0], 
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
#[derive(Clone, Serialize, Deserialize)]
pub struct BareEdgeVersion4 {
    label: Option<String>,
    from: u32,
    to: u32,
    index: u32,
}

impl Default for BareEdgeVersion4 {
    fn default() -> Self {
        BareEdgeVersion4 {
            label: None,
            from: 0,
            to: 0,
            index: 0,
        }
    }
}