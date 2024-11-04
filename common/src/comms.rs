/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// communications enums
// includes Server -> Client, Client -> Server, Webserver -> Game Engine, Game Engine -> Webserver
// impls where applicable sorted to the bottom of the file

use crate::protocol::ProtocolRoot;
use crate::protocol::Protocol;
use crate::protocol::DecodeError;


#[derive(Debug, ProtocolRoot, PartialEq)]
pub enum ClientMessage { // client -> server
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8), // the test message. See ServerMessage.
    Connect(String, String), // connect with your nickname and the password respectively. doesn't let you place your castle yet.
    // passwords are, like in MMOSG, used for various things: they can grant entry into a server, they can assign your team, etc. In io games they are usually ignored.
    PlacePiece(f32, f32, u16), // x, y, type
    // attempt to place an object
    // before the client can place anything else, it must place a castle (type 1). this is the only time in the game that a client can place an object in neutral territory.
    // obviously it's not possible to place a castle in enemy territory
    StrategyPointAdd(u32, u16, f32, f32), // (id, index, x, y) insert a point to a strategy path at an index
    StrategyPointUpdate(u32, u16, f32, f32), // (id, index, x, y) move a point on a strategy path
    StrategyRemove(u32, u16), // (id, index) remove a node from a strategy
    StrategyClear(u32), // (id) clear a strategy
    // if any Strategy commands are sent referencing nonexistent nodes on a strategy, or StrategyPointUpdate is sent referencing a non-point strategy node (such
    // as a teleportal entrance), the server will simply ignore them. This may be a problem in the future.
    StrategySetEndcapRotation(u32, f32), // (id, r) set the strategy endcap for an object to a rotation
    StrategyTargetAdd(u32, u32), // (id, target) add a piece to id's strategy
    GunState(u32, u8) // (id, state) change the state of a gun. 0 = disable, 1 = enable.
}

// upon connecting, the server immediately sends the client Test("EXOSPHERE", 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION)
// if the client can successfully decode the message, and VERSION matches the client version, the game may proceed. Otherwise, the client will immediately drop the connection.
// to signify that the test passed, the client will send a Test() with the same values and the client version. If the server can successfully decode them, and the version the
// client sends matches VERSION, the game may proceed. Otherwise, the server will abort the connection.
// this exchange prevents old, underprepared, or incompatible clients from connecting to a game.
// If a client attempts to do anything before protocol verification, it will be kicked off the server.

#[derive(Debug, ProtocolRoot, Clone)]
pub enum ServerMessage { // server -> client
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8), // the test message. see above blurb.
    GameState(u8, u16, u16), // game state, stage tick, stage total time
    // this is just 5 bytes - quite acceptable to send to every client every frame, much lower overhead than even ObjectMove.
    // the first byte is bitbanged. bit 1 is io mode enable - in io mode, anyone can place a castle at any time. bit 2 is waiting/playing (1 = playing, 0 = waiting): in wait mode,
    // castles can be placed, and the gameserver begins the game when >=2 castles have been waiting for a certain duration.
    // wait mode does not exist in io mode; if bits 1 and 2 are set, something's wrong. Bit 3 controls if the mode is "move ships" (1) or "play" (0) - in
    // "move ships" mode, people can set and modify the paths their ships will follow, and in play mode the ships will move along those paths.
    // In play mode, castles that wish to do so may also "possess" a ship, controlling its motion in real time; this is the replacement for MMOSG's RTFs.
    // Bits 4-8 are reserved.
    Metadata(u64, f32, f32, u8), // send whatever data (id, board width x height, slot) the client needs to begin rendering the gameboard
    // this also tells the client that it was accepted (e.g. got the right password); getting the password _wrong_ would abort the connection
    // slot tells the client what position it's occupying. 0 = spectating, 1 = free agent, 2-255 = teams.
    ObjectCreate(f32, f32, f32, u64, u32, u16), // x, y, a, owner, id, type: inform the client of an object.
    ObjectMove(u32, f32, f32, f32), // id, x, y, a
    ObjectTrajectoryUpdate(u32, f32, f32, f32, f32, f32, f32), // id, x, y, a, xv, yv, av
    DeleteObject(u32), // id
    StrategyCompletion(u32, u16), // (id, remaining) a node in a strategy has been fulfilled, and this is the number of strategy nodes remaining!
    // this serves as a sort of checksum; if the number of strategy nodes remaining here is different from the number of strategy nodes remaining in the client
    // the client knows there's something wrong and can send StrategyClear to attempt to recover.
    PlayerData(u64, String, u8), // (id, banner, slot): data on another player
    YouLose, // you LOST!
    Winner(u64), // id of the winner. if 0, it was a tie.
    Territory(u32, f32), // set the territory radius for a piece
    Fabber(u32, f32), // set the fabber radius for a piece
    Disconnect,
    Money(u64, u32), // set the money amount for a client
    // in the future we may want to be able to see the money of our allies, so the id tag could be useful
    Explosion(f32, f32, f32, f32), // x, y, radius, damage: an explosion happened! the client should render it for one frame and then kill it
    Health(u32, f32), // (id, health): tell a player about the current health of one of its pieces
}