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

use crate::pathfollower::PathNode;
use crate::types::PieceType;
use crate::PieceId;
use crate::PlayerId;
use bitcode::{Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq)]
pub enum StrategyPathModification {
    Insert(PieceId, u16, PathNode),
    Clear(PieceId),
    Set(PieceId, u16, PathNode),
    Delete(PieceId, u16),
}

#[derive(Debug, Encode, Decode, PartialEq)]
pub enum ObjectSpecialPropertySet {
    GunState(bool),
}

#[derive(Debug, Encode, Decode, PartialEq)]
pub enum ClientMessage {
    // client -> server
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8), // the test message. See ServerMessage.
    Connect {
        nickname: String,
    },                          // connect to the server. doesn't let you place your castle yet.
    // if this works immediately, the server will send Metadata(...), which signals the user to begin playing.
    // the server can also send a challenge: PasswordChallenge or TeamChallenge. In the case of PasswordChallenge,
    // the client will send TryPassword. In the case of TeamChallenge, the client will send TryTeam. In either case,
    // the client may also send TrySpectate. After this exchange the server will either send Metadata or Reject.
    TryPassword {
        password: String,
    },
    TryTeam {
        team_number: u8, // 0 for "I want to play as a free agent", which may or may not be allowed
        password: String,
    },
    TrySpectate, // I don't want to play, I just want to watch
    PlacePiece {
        x: f32,
        y: f32,
        tp: PieceType,
    }, // x, y, type
    // attempt to place an object
    // before the client can place anything else, it must place a castle (type 1). this is the only time in the game that a client can place an object in neutral territory.
    // obviously it's not possible to place a castle in enemy territory
    Strategy {
        evt: StrategyPathModification,
    },
    Special {
        id: PieceId,
        evt: ObjectSpecialPropertySet,
    },
}

// upon connecting, the server immediately sends the client Test("EXOSPHERE", 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION)
// if the client can successfully decode the message, and VERSION matches the client version, the game may proceed. Otherwise, the client will immediately drop the connection.
// to signify that the test passed, the client will send a Test() with the same values and the client version. If the server can successfully decode them, and the version the
// client sends matches VERSION, the game may proceed. Otherwise, the server will abort the connection.
// this exchange prevents old, underprepared, or incompatible clients from connecting to a game.
// If a client attempts to do anything before protocol verification, it will be kicked off the server.

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq)]
pub enum Stage {
    Playing,
    Waiting,
    MoveShips,
}

impl Stage {
    pub fn get_str(self) -> &'static str {
        match self {
            Self::Playing => "PLAYING",
            Self::Waiting => "WAITING",
            Self::MoveShips => "MOVE SHIPS",
        }
    }
}

/// ServerMessage represents a data frame sent from the Bevy gameserver to the websocket client.
#[derive(Debug, Encode, Decode, Clone)]
pub enum ServerMessage {
    /// The standard test frame. This is sent to the client on startup, and an exactly equal one is sent back;
    /// if ever they don't match, the connection will not proceed.
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8),
    GameState {
        stage: Stage,
        stage_duration: u16,
        tick_in_stage: u16,
    },
    /// The startup message. Receiving Metadata instructs the client that
    /// it has been accepted and should show the user the gameboard.
    Metadata {
        id: PlayerId,
        slot: u8,
        board_width: f32,
        board_height: f32,
    },
    /// A password is required on this server. The client should send a password attempt
    /// or request to spectate.
    PasswordChallenge,
    /// A team is required on this server. The client should send a team ID and password
    /// or request to spectate.
    TeamChallenge {
        available: Vec<(String, u8)>, // (team name, slot number)
    },
    /// The client failed either the PasswordChallenge or TeamChallenge, and should
    /// either request to spectate or disconnect.
    Reject,
    /// A piece was added to the gameboard.
    ObjectCreate {
        id: PieceId,
        x: f32,
        y: f32,
        a: f32,
        owner: PlayerId,
        tp: PieceType,
    },
    /// A piece on the gameboard was moved.
    ObjectMove { id: PieceId, x: f32, y: f32, a: f32 },
    /// A piece on the gameboard was deleted.
    DeleteObject { id: PieceId },
    /// A node in a strategy path has been fulfilled: this tells the client
    /// how many nodes are left, as a sort of checksum.
    StrategyCompletion { id: PieceId, remaining: u16 },
    /// Information about a player.
    PlayerData {
        id: PlayerId,
        nickname: String,
        slot: u8,
    },
    /// You LOST!
    YouLose,
    /// Somebody won! Sends id 0 (SYSTEM) for a tie.
    Winner { id: PlayerId },
    /// Establish a territory influence around an object.
    Territory { id: PieceId, radius: f32 },
    /// Establish a fabber influence around an object.
    Fabber { id: PieceId, radius: f32 },
    /// The client will be disconnected.
    Disconnect,
    /// The client has this amount of cash on hand!
    Money { id: PlayerId, amount: u32 },
    /// Something blew up. The explosion should be cleared
    /// after no more than one frame.
    Explosion {
        x: f32,
        y: f32,
        radius: f32,
        damage: f32,
    },
    /// Inform the client about the health of one of its pieces.
    Health { id: PieceId, health: f32 },
    /// A laser was cast. Like explosions, these should only be rendered
    /// for one frame.
    LaserCast {
        caster: PieceId,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
    },
}
