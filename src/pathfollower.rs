// the PathFollower class
// at the moment all it does is curate a list of Nodes and an EndNode.
use bevy::prelude::Entity;


#[derive(Copy, Clone, Debug)]
pub enum PathNode {
    StraightTo(f32, f32),
    Teleportal(Entity, Entity) // entrypoint, exitpoint
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
    end : EndNode
    // empty paths are "unlinked"; unlinked objects will not attempt to move at all.
    // path following will never clear the last node in a path (this is the endcap node, and often has extra metadata associated with it).
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
            end : EndNode::None
        }
    }

    pub fn insert_point(&mut self, _index : u16, x : f32, y : f32) {
        let index : usize = _index as usize;
        if index <= self.nodes.len() {
            self.nodes.insert(index, PathNode::StraightTo(x, y));
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