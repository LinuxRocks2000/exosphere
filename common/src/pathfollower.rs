/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// the PathFollower class
// at the moment all it does is curate a list of Nodes

use serde_derive::{ Serialize, Deserialize };
use crate::PieceId;


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PathNode {
    StraightTo(f32, f32),
    Target(PieceId),
    Rotation(f32, u16) // angle, duration
}


use std::collections::VecDeque;


pub struct PathFollower { // follow a path.
    nodes : VecDeque<PathNode>
    // empty paths are "unlinked"; unlinked objects will not attempt to move at all.
    // path following will never clear the last node in a path (this is the endcap node).
}


impl PathFollower {
    pub fn get_next(&self) -> Option<PathNode> {
        if self.nodes.len() > 0 {
            self.nodes.get(0).copied()
        }
        else {
            None
        }
    }

    pub fn len(&self) -> Result<u16, impl std::error::Error> {
        self.nodes.len().try_into()
    }

    pub fn endex(&self) -> Result<u16, impl std::error::Error> { // return the index needed to add a new node to this path
        self.nodes.len().try_into()
    }

    pub fn bump(&mut self) -> Result<bool, impl std::error::Error> { // if it's determined that we've completed a goal, truncate and go to the next one
        // returns true if the node was actually bumped, false otherwise (it won't bump if doing so would unlink this piece)
        if match self.len() { Ok(v) => v, Err(e) => { return Err(e); } } > 1 {
            #[cfg(feature="server")]
            { // if we're the server, run the code that makes rotation durations work
                if let Some(PathNode::Rotation(_, dur)) = self.nodes.get_mut(0) {
                    if *dur > 0 {
                        *dur -= 1;
                        return Ok(false);
                    }
                }
            }
            self.nodes.pop_front();
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn start(x : f32, y : f32) -> Self {
        Self {
            nodes : VecDeque::from([PathNode::StraightTo(x, y)])
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
    }

    pub fn remove_node(&mut self, index : u16) {
        if (index as usize) < self.nodes.len() {
            self.nodes.remove(index as usize);
        }
    }

    pub fn iter<'a>(&'a self) -> PathIter<'a> {
        PathIter {
            path : self,
            index : 0
        }
    }

    pub fn get(&self, ind : usize) -> Option<PathNode> {
        if ind < self.nodes.len() {
            Some(self.nodes[ind])
        }
        else {
            None
        }
    }

    pub fn get_last(&self) -> Option<PathNode> {
        self.get(self.nodes.len() - 1)
    }
}


pub struct PathIter<'a> {
    path : &'a PathFollower,
    index : usize
}


impl<'a> Iterator for PathIter<'a> {
    type Item = PathNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.path.get(self.index - 1)
    }
}