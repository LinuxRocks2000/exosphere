/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// the PathFollower class
// at the moment all it does is curate a list of Nodes and an EndNode.
use bevy::prelude::Entity;


#[derive(Copy, Clone, Debug)]
pub enum PathNode {
    StraightTo(f32, f32),
    Target(Entity)
}

#[derive(Copy, Clone)]
pub enum EndNode {
    None,
    Rotation(f32)
}


use std::collections::VecDeque;
use bevy::prelude::Component;


#[derive(Component)]
pub struct PathFollower { // follow a path.
    nodes : VecDeque<PathNode>,
    end : EndNode,
    can_track : bool // can this pathfollower be used to track an object on the board?
    // empty paths are "unlinked"; unlinked objects will not attempt to move at all.
    // path following will never clear the last node in a path (this is the endcap node, and often has endnode data associated with it).
}


impl PathFollower {
    pub fn get_next(&self) -> Option<PathNode> {
        self.nodes.get(0).copied()
    }

    pub fn get_endcap(&self) -> EndNode {
        self.end
    }

    pub fn bump(&mut self) -> bool { // if it's determined that we've completed a goal, truncate and go to the next one
        if self.nodes.len() > 1 {
            self.nodes.pop_front();
            true
        }
        else {
            false
        }
    }

    pub fn start(x : f32, y : f32) -> Self {
        Self {
            nodes : VecDeque::from([PathNode::StraightTo(x, y)]),
            end : EndNode::None,
            can_track : false
        }
    }

    pub fn with_tracking(mut self) -> Self {
        self.can_track = true;
        self
    }

    pub fn insert_point(&mut self, _index : u16, x : f32, y : f32) {
        let index : usize = _index as usize;
        if index <= self.nodes.len() {
            self.nodes.insert(index, PathNode::StraightTo(x, y));
        }
    }

    pub fn insert_target(&mut self, _index : u16, t : Entity) {
        if self.can_track {
            let index : usize = _index as usize;
            if index <= self.nodes.len() {
                self.nodes.insert(index, PathNode::Target(t));
            }
        }
    }

    pub fn update_point(&mut self, _index : u16, x : f32, y : f32) {
        let index : usize = _index as usize;
        if index <= self.nodes.len() {
            if let Some(PathNode::StraightTo(_, _)) = self.nodes.get(index) {
                self.nodes[index] = PathNode::StraightTo(x, y);
            }
        }
    }

    pub fn len(&self) -> u16 {
        self.nodes.len().try_into().expect("error: length of strategy is greater than u16 limit! this is not possible")
    }

    pub fn clear(&mut self) {
        self.nodes.clear()
    }

    pub fn remove_node(&mut self, index : u16) {
        if (index as usize) < self.nodes.len() {
            self.nodes.remove(index as usize);
        }
    }

    pub fn set_endcap_rotation(&mut self, r : f32) {
        self.end = EndNode::Rotation(r);
    }
}