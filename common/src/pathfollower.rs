/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// the PathFollower class
// at the moment all it does is curate a list of Nodes and an EndNode.

use serde_derive::{ Serialize, Deserialize };
use crate::PieceId;


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PathNode {
    StraightTo(f32, f32),
    Target(PieceId),
    Rotation(f32)
}


use std::collections::VecDeque;


pub struct PathFollower { // follow a path.
    nodes : VecDeque<PathNode>,
    end : Option<PathNode>
    // empty paths are "unlinked"; unlinked objects will not attempt to move at all.
    // path following will never clear the last node in a path (this is the endcap node, and often has endnode data associated with it).
}


impl PathFollower {
    pub fn get_next(&self) -> Option<PathNode> {
        if self.nodes.len() > 0 {
            self.nodes.get(0).copied()
        }
        else {
            self.end
        }
    }

    pub fn len(&self) -> Result<u16, impl std::error::Error> {
        (self.nodes.len() + match self.end {
            Some(_) => 1,
            None => 0
        }).try_into()
    }

    pub fn bump(&mut self) -> Result<bool, impl std::error::Error> { // if it's determined that we've completed a goal, truncate and go to the next one
        // returns true if the node was actually bumped, false otherwise (it won't bump if doing so would unlink this piece)
        if match self.len() { Ok(v) => v, Err(e) => { return Err(e); } } > 0 {
            self.nodes.pop_front();
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn start(x : f32, y : f32) -> Self {
        Self {
            nodes : VecDeque::from([PathNode::StraightTo(x, y)]),
            end : None
        }
    }

    pub fn insert_node(&mut self, index : u16, node : PathNode) {
        let index = index as usize;
        if index <= self.nodes.len() {
            self.nodes.insert(index, node);
        }
    }

    pub fn update_node(&mut self, index : u16, node : PathNode) {
        let index = index as usize;
        if index <= self.nodes.len() {
            self.nodes[index] = node;
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.end = None;
    }

    pub fn remove_node(&mut self, index : u16) {
        if (index as usize) < self.nodes.len() {
            self.nodes.remove(index as usize);
        }
    }

    pub fn set_endcap(&mut self, end : Option<PathNode>) {
        self.end = end;
    }
}