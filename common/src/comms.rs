/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// common communications enums
// includes Server -> Client and Client -> Server
// impls where applicable sorted to the bottom of the file

use serde_derive::{Serialize, Deserialize};
use crate::pathfollower::PathNode;
use crate::PieceId;
use crate::PlayerId;
use crate::types::PieceType;


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ClientMessage { // client -> server
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8), // the test message. See ServerMessage.
    Connect { nickname : String, password : String }, // connect to the server. doesn't let you place your castle yet.
    // passwords are, like in MMOSG, used for various things: they can grant entry into a server, they can assign your team, etc. In io games they are usually ignored.
    PlacePiece { x : f32, y : f32, tp : PieceType }, // x, y, type
    // attempt to place an object
    // before the client can place anything else, it must place a castle (type 1). this is the only time in the game that a client can place an object in neutral territory.
    // obviously it's not possible to place a castle in enemy territory
    StrategyInsert { piece : PieceId, index : u16, node : PathNode },
    StrategySet { piece : PieceId, index : u16, node : PathNode },
    StrategyDelete { piece : PieceId, index : u16 },
    StrategyClear { piece : PieceId },
    GunState { piece : PieceId, enabled : bool } // change the state of a gun.
}

// upon connecting, the server immediately sends the client Test("EXOSPHERE", 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION)
// if the client can successfully decode the message, and VERSION matches the client version, the game may proceed. Otherwise, the client will immediately drop the connection.
// to signify that the test passed, the client will send a Test() with the same values and the client version. If the server can successfully decode them, and the version the
// client sends matches VERSION, the game may proceed. Otherwise, the server will abort the connection.
// this exchange prevents old, underprepared, or incompatible clients from connecting to a game.
// If a client attempts to do anything before protocol verification, it will be kicked off the server.

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Stage {
    Playing,
    Waiting,
    MoveShips
}


impl Stage {
    pub fn get_str(self) -> &'static str {
        match self {
            Self::Playing => "PLAYING",
            Self::Waiting => "WAITING",
            Self::MoveShips => "MOVE SHIPS"
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage { // server -> client
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8), // the test message. see above blurb.
    GameState { stage : Stage, stage_duration : u16, tick_in_stage : u16 },
    Metadata { id : PlayerId, slot : u8, board_width : f32, board_height : f32 }, // send whatever data (id, board width x height, slot) the client needs to begin rendering the gameboard
    // this also tells the client that it was accepted (e.g. got the right password); getting the password _wrong_ would abort the connection
    // slot tells the client what position it's occupying. 0 = spectating, 1 = free agent, 2-255 = teams.
    ObjectCreate { id : PieceId, x : f32, y : f32, a : f32, owner : PlayerId, tp : PieceType },
    ObjectMove { id : PieceId, x : f32, y : f32, a : f32 },
    DeleteObject { id : PieceId },
    StrategyCompletion { id : PieceId, remaining : u16 }, // a node in a strategy has been fulfilled, and this is the number of strategy nodes remaining!
    // this serves as a sort of checksum; if the number of strategy nodes remaining here is different from the number of strategy nodes remaining in the client
    // the client knows there's something wrong and can send StrategyClear to attempt to recover.
    PlayerData { id : PlayerId, nickname : String, slot : u8},
    YouLose, // you LOST!
    Winner { id : PlayerId }, // id of the winner. if 0, it was a tie.
    Territory { id : PieceId, radius : f32}, // set the territory radius for a piece
    Fabber { id : PieceId, radius : f32}, // set the fabber radius for a piece
    Disconnect,
    Money { id : PlayerId, amount : u32}, // set the money amount for a client
    // in the future we may want to be able to see the money of our allies, so the id tag could be useful
    Explosion { x : f32, y : f32, radius : f32, damage : f32}, // an explosion happened! the client should render it for one frame and then kill it
    Health { id : PieceId, health : f32 }, // tell a player about the current health of one of its pieces
    LaserCast { caster : PieceId, from_x : f32, from_y : f32, to_x : f32, to_y : f32 }
}