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
#![allow(dead_code)]
use std::{collections::{HashMap, HashSet}};

use druid::{WidgetPod, EventCtx, Command, Target, Vec2, Affine};
use serde::{Deserialize, Serialize};
use vm_force_graph_rs::{Node, NodeData, DefaultNodeIdx, EdgeData};
use petgraph::{stable_graph::StableUnGraph, visit::{EdgeRef, IntoEdgeReferences}};

use crate::{vmnode::VMNode, vimmapper::VimMapper, VMTab, constants::SET_REGISTER, vmconfig::VMConfigVersion4, vminput::KeybindMode};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VMGraphClip {
    nodes: HashMap<u32, VMNode>,
    graph: StableUnGraph<Node<u32>, EdgeData<u32>>,
    pub root_node: Option<DefaultNodeIdx>,
}

impl VMGraphClip {
    pub fn dispatch(ctx: &mut EventCtx, mapper: &VimMapper, node_set: &HashSet<DefaultNodeIdx>, root: DefaultNodeIdx, _register_name: &String) {
        let mut graph_clip = VMGraphClip {
            nodes: HashMap::new(),
            graph: StableUnGraph::default(),
            root_node: None,
        };

        let root_pos = Vec2::new(mapper.graph.get_graph()[root].data.x, mapper.graph.get_graph()[root].data.y);
        let root_angle = root_pos.atan2();
        let rotation = Affine::rotate(root_angle).inverse();

        let mut trans_map: HashMap<DefaultNodeIdx, DefaultNodeIdx> = HashMap::new();

        for fg_idx in node_set {
            let node = &mapper.get_force_graph().get_graph()[*fg_idx];
            graph_clip.nodes.insert(node.data.user_data, mapper.get_nodes().get(&node.data.user_data).unwrap().clone());
            let new_idx = graph_clip.get_graph_mut().add_node(node.clone());
            let mut new_node_pos = Vec2::new(
                graph_clip.get_graph()[new_idx].data.x,
                graph_clip.get_graph()[new_idx].data.y
            );

            new_node_pos -= root_pos;
            new_node_pos = Vec2::from(((rotation * new_node_pos.to_point()).x, (rotation * new_node_pos.to_point()).y));
            graph_clip.get_graph_mut()[new_idx].data.x = new_node_pos.x;
            graph_clip.get_graph_mut()[new_idx].data.y = new_node_pos.y;
            trans_map.insert(node.index(), new_idx);
        }
        graph_clip.root_node = Some(*trans_map.get(&root).unwrap());
        for fg_idx in node_set {
            let mut edges = mapper.graph.get_graph().edges(*fg_idx).clone();
            while let Some(edge) = edges.next() {
                if let Some(source_idx) = trans_map.get(&edge.source()) {
                    if let Some(target_idx) = trans_map.get(&edge.target()) {
                        if graph_clip.get_graph().contains_node(*source_idx) && graph_clip.get_graph().contains_node(*target_idx) {
                            graph_clip.get_graph_mut().update_edge(*source_idx, *target_idx, EdgeData {user_data: 0});
                        }
                    }
                }
            }
        }
        ctx.submit_command(Command::new(SET_REGISTER,
            ("0".to_string(), graph_clip),
            Target::Global,
        ));
    }

    pub fn get_root_node(&self) -> &Option<DefaultNodeIdx> {
        return &self.root_node;
    }

    pub fn get_root_node_mut(&self) -> Option<DefaultNodeIdx> {
        return self.root_node;
    }

    pub fn set_root_node(&mut self, root_node: DefaultNodeIdx) {
        self.root_node = Some(root_node);
    }

    pub fn get_graph(&self) -> &StableUnGraph<Node<u32>, EdgeData<u32>> {
        return &self.graph;
    }

    pub fn get_node_map(&self) -> &HashMap<u32, VMNode> {
        return &self.nodes;
    }

    pub fn get_graph_mut(&mut self) -> &mut StableUnGraph<Node<u32>, EdgeData<u32>> {
        return &mut self.graph;
    }

    pub fn get_node_map_mut(&mut self) -> &mut HashMap<u32, VMNode> {
        return &mut self.nodes;
    }

    pub fn append_node_clip(&self, target: &mut VimMapper, target_idx: Option<u32>, _register: String) {
        let mut trans_map: HashMap<DefaultNodeIdx, DefaultNodeIdx> = HashMap::new(); 
        if let Some(target_idx) = target_idx {
            let target_node = &target.graph.get_graph()[target.nodes.get(&target_idx).unwrap().fg_index.unwrap()];
            let target_node_pos = Vec2::new(target_node.x(), target_node.y());
            let angle = target_node_pos.atan2();
            let rotation = Affine::rotate(angle);
            let new_node_offset = target_node_pos + Vec2::new(
                (rotation * Vec2::new(target_node.data.repel_distance, target_node.data.repel_distance).to_point()).x,
                (rotation * Vec2::new(target_node.data.repel_distance, target_node.data.repel_distance).to_point()).y,
            );
            for old_fg_index in self.graph.node_indices() {
                let node = self.graph[old_fg_index].clone();
                let new_index = target.increment_node_idx();
                let new_node_pos = (rotation * Vec2::new(node.data.x, node.data.y).to_point()) + new_node_offset;
                let new_fg_index = target.graph.add_node(NodeData {
                    x: new_node_pos.x,
                    y: new_node_pos.y,
                    mass: node.data.mass,
                    repel_distance: node.data.repel_distance,
                    is_anchor: node.data.is_anchor,
                    user_data: new_index,
                });
                let mut vm_node = self.nodes.get(&node.data.user_data).unwrap().clone();
                if let Some(mark) = vm_node.mark.clone() {
                    if mark.contains(char::is_numeric) {
                        vm_node.mark = None;
                    }
                }
                vm_node.index = new_index;
                vm_node.is_active = false;
                vm_node.fg_index = Some(new_fg_index);
                target.nodes.insert(new_index, vm_node);
                trans_map.insert(old_fg_index, new_fg_index);
            }
            for edge in self.graph.edge_references() {
                target.graph.add_edge(*trans_map.get(&edge.source()).unwrap(), *trans_map.get(&edge.target()).unwrap(), EdgeData { user_data: 0 });
            }
            target.graph.add_edge(target.nodes.get(&target_idx).unwrap().fg_index.unwrap(), *trans_map.get(&self.root_node.unwrap()).unwrap(), EdgeData{ user_data: 0 });
            target.build_target_list_from_neighbors(target_idx);
        } else if let Some(root_node) = self.root_node {
            let external_node = target.add_external_node(self.nodes.get(&self.graph[root_node].data.user_data).unwrap().label.clone()).unwrap();
            let external_fg_index = target.get_nodes().get(&external_node).unwrap().fg_index.unwrap();
            target.set_node_as_active(external_node);
            trans_map.insert(root_node, target.get_nodes().get(&external_node).unwrap().fg_index.unwrap());
            for old_fg_index in self.graph.node_indices() {
                if old_fg_index != root_node {
                    let node = self.graph[old_fg_index].clone();
                    let new_index = target.increment_node_idx();
                    let new_fg_index = target.graph.add_node(NodeData {
                        x: node.data.x,
                        y: node.data.y,
                        mass: node.data.mass,
                        repel_distance: node.data.repel_distance,
                        is_anchor: node.data.is_anchor,
                        user_data: new_index,
                    });
                    let mut vm_node = self.nodes.get(&node.data.user_data).unwrap().clone();
                    vm_node.index = new_index;
                    vm_node.is_active = false;
                    vm_node.fg_index = Some(new_fg_index);
                    target.nodes.insert(new_index, vm_node);
                    trans_map.insert(old_fg_index, new_fg_index);
                }
            }
            for edge in self.graph.edge_references() {
                if edge.source() == root_node {
                    target.graph.add_edge(external_fg_index, *trans_map.get(&edge.target()).unwrap(), EdgeData { user_data: 0 });
                } else if edge.target() == root_node {
                    target.graph.add_edge(*trans_map.get(&edge.source()).unwrap(), external_fg_index, EdgeData { user_data: 0 });
                } else {
                    target.graph.add_edge(*trans_map.get(&edge.source()).unwrap(), *trans_map.get(&edge.target()).unwrap(), EdgeData { user_data: 0 });
                }
            }
            target.build_target_list_from_neighbors(external_node);
            target.input_manager.set_keybind_mode(KeybindMode::Move);
        }
        target.animating = true;
    }

    pub fn init_tab_with_clip(&self, config: VMConfigVersion4) -> VMTab {
            let root_node = self.root_node.unwrap();
            let mut trans_map: HashMap<DefaultNodeIdx, DefaultNodeIdx> = HashMap::new(); 
            let mut target = VimMapper::new(config);
            let label = self.nodes.get(&self.graph[root_node].data.user_data).unwrap().label.clone();
            target.nodes.get_mut(&0).unwrap().label = label.clone();
            let new_root_node = 0;
            let new_root_fg_index = target.get_nodes().get(&0).unwrap().fg_index.unwrap();
            target.set_node_as_active(new_root_node);
            trans_map.insert(root_node, target.get_nodes().get(&new_root_node).unwrap().fg_index.unwrap());
            for old_fg_index in self.graph.node_indices() {
                if old_fg_index != root_node {
                    let node = self.graph[old_fg_index].clone();
                    let new_index = target.increment_node_idx();
                    let new_fg_index = target.graph.add_node(NodeData {
                        x: node.data.x,
                        y: node.data.y,
                        mass: node.data.mass,
                        repel_distance: node.data.repel_distance,
                        is_anchor: node.data.is_anchor,
                        user_data: new_index,
                    });
                    let mut vm_node = self.nodes.get(&node.data.user_data).unwrap().clone();
                    vm_node.index = new_index;
                    vm_node.is_active = false;
                    vm_node.fg_index = Some(new_fg_index);
                    target.nodes.insert(new_index, vm_node);
                    trans_map.insert(old_fg_index, new_fg_index);
                }
            }
            for edge in self.graph.edge_references() {
                if edge.source() == root_node {
                    target.graph.add_edge(new_root_fg_index, *trans_map.get(&edge.target()).unwrap(), EdgeData { user_data: 0 });
                } else if edge.target() == root_node {
                    target.graph.add_edge(*trans_map.get(&edge.source()).unwrap(), new_root_fg_index, EdgeData { user_data: 0 });
                } else {
                    target.graph.add_edge(*trans_map.get(&edge.source()).unwrap(), *trans_map.get(&edge.target()).unwrap(), EdgeData { user_data: 0 });
                }
            }
            target.build_target_list_from_neighbors(new_root_node);
            VMTab {
                vm: WidgetPod::new(target),
                tab_name: label.clone(),
            }
    }
}