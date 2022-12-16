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

use druid::{Vec2, Data};
use druid::kurbo::TranslateScale;
use vm_force_graph::{ForceGraph, DefaultNodeIdx, NodeData, EdgeData};
use serde::{Serialize, Deserialize};

use crate::constants::*;

use crate::vimmapper::NodeRenderMode;
use crate::vmnode::{VMNode, VMEdge, VMNodeEditor};
use crate::{vmconfig::VMConfigVersion4, vimmapper::VimMapper};


//A boiled-down struct to hold the essential data to serialize and deserialize a graph sheet. Used to
// enable the app state to be saved to disk as a .vmd file.
#[derive(Serialize, Deserialize)]
pub struct VMSaveVersion4 {
    file_version: String, 
    nodes: HashMap<u16, BareNodeVersion4>,
    edges: HashMap<u16, BareEdgeVersion4>,
    node_idx_count: u16,
    edge_idx_count: u16,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

#[derive(Data, PartialEq, Clone, Debug)]
pub enum VMSaveState {
    NoSheetOpened,
    NoSave,
    UnsavedChanges,
    // SaveAsInProgressFileExists,
    SaveAsInProgress,
    // SaveAsInProgressFileExistsThenQuit,
    SaveAsInProgressThenQuit,
    // SaveAsInProgressFileExistsThenNew,
    SaveAsInProgressThenNew,
    SaveAsInProgressThenOpen,
    // SaveInProgress,
    // SaveInProgressThenQuit,
    Saved,
}

#[derive(Serialize, Deserialize)]
pub struct VMSaveNoVersion {
    nodes: HashMap<u16, BareNodeVersion4>,
    edges: HashMap<u16, BareEdgeVersion4>,
    node_idx_count: u16,
    edge_idx_count: u16,
    translate: (f64, f64),
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

impl VMSaveNoVersion {
    fn convert_to_current(&mut self) -> VMSaveVersion4 {
        VMSaveVersion4 {
            file_version: CURRENT_SAVE_FILE_VERSION.to_string(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            node_idx_count: self.node_idx_count,
            edge_idx_count: self.edge_idx_count,
            translate: self.translate,
            scale: self.scale,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
        }
    }
}

pub struct VMSaveSerde;

impl VMSaveSerde {
    //Instantiates a new VimMapper struct from a deserialized VMSave. The ForceGraph is created from scratch
    // and no fg_index values are guaranteed to persist from session to session.
    pub(crate) fn from_save(save: VMSaveVersion4, config: VMConfigVersion4) -> VimMapper {
        let mut graph = <ForceGraph<u16, u16>>::new(DEFAULT_SIMULATION_PARAMTERS);
        let mut nodes: HashMap<u16, VMNode> = HashMap::with_capacity(50);
        let mut edges: HashMap<u16, VMEdge> = HashMap::with_capacity(100);
        for (_k ,v) in save.nodes {
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
                edges: v.edges, 
                index: v.index, 
                fg_index: fg_index, 
                // pos: Vec2::new(v.pos.0, v.pos.1), 
                // container: VMNodeLayoutContainer::new(v.index), 
                mark: v.mark,
                anchored: v.anchored,
                mass: v.mass,
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
            is_focused: true,
            target_node_list: vec![],
            target_node_idx: None,
            node_editor: VMNodeEditor::new(),
            is_hot: true,
            config,
            node_render_mode: NodeRenderMode::AllEnabled,
            ..Default::default()
        };
        vm.set_node_as_active(0);
        // vm.build_target_list_from_neighbors(0);
        // vm.cycle_target_forward();
        vm
    }

    //Instantiates a serializable VMSave from the VimMapper struct. All ForceGraph data is discarded and
    // must be recreated when the VMSave is deserialized and instantiated into a VimMapper struct
    pub(crate) fn to_save(vm: &VimMapper) -> VMSaveVersion4 {
        let mut nodes: HashMap<u16, BareNodeVersion4> = HashMap::with_capacity(50);
        let mut edges: HashMap<u16, BareEdgeVersion4> = HashMap::with_capacity(100);
        vm.get_nodes().iter().for_each(|(index, node)| {
            let pos = vm.get_node_pos(*index);
            nodes.insert(*index, BareNodeVersion4 {
                label: node.label.clone(),
                edges: node.edges.clone(),
                index: node.index,
                // pos: (node.pos.x, node.pos.y),
                pos: (pos.x, pos.y),
                is_active: false,
                targeted_internal_edge_idx: None,
                mark: node.mark.clone(),
                mass: node.mass,
                anchored: node.anchored,
            });
        });
        vm.get_edges().iter().for_each(|(index, edge)| {
            edges.insert(*index, BareEdgeVersion4 {
                label: None,
                from: edge.from,
                to: edge.to,
                index: *index,
            });
        });
        let save = VMSaveVersion4 {
            file_version: CURRENT_SAVE_FILE_VERSION.to_string(),
            nodes: nodes,
            edges: edges,
            node_idx_count: vm.get_node_idx_count(),
            edge_idx_count: vm.get_edge_idx_count(),
            translate: (vm.get_translate().as_tuple().0.x, vm.get_translate().as_tuple().0.y),
            scale: vm.get_scale().as_tuple().1,
            offset_x: vm.get_offset_x(),
            offset_y: vm.get_offset_y(),
        };
        save
    }

    pub(crate) fn load(path: String) -> Result<(VMSaveVersion4, PathBuf), String> {
        if let Ok(string) = fs::read_to_string(path.clone()) {
            if let Ok(path) = Path::new(&path.clone()).canonicalize() {
                if let Ok(save) = serde_json::from_str::<VMSaveVersion4>(string.as_str()) {
                    return Ok((save, path));
                } else if let Ok(mut save) = serde_json::from_str::<VMSaveNoVersion>(string.as_str()) {
                    return Ok((save.convert_to_current(), path));
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

    pub(crate) fn save(save: &VMSaveVersion4, path: PathBuf) -> Result<String, String> {
        #[cfg(debug_assertions)]
        {
            println!("Saving file to {}", path.display());
        }
        if let Ok(string) = serde_json::to_string::<VMSaveVersion4>(save) {
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
    edges: Vec<u16>,
    index: u16,
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
#[derive(Clone, Serialize, Deserialize)]
pub struct BareEdgeVersion4 {
    label: Option<String>,
    from: u16,
    to: u16,
    index: u16,
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