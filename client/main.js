/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

window.onload = () => {
    var connection = new Connection("ws://localhost:3000/game");
    var protocol = connection.load_protocol(OUTGOING_PROTOCOL);
    connection.onclose = () => {
        alert("connection broken.");
    };
    connection.onopen = () => {
        protocol.Test("hello, world", 0, 12.345);
    };
    connection.onMessage("Test", (s, n, f) => {
        console.log("got test string " + s);
        console.log("test u16 " + n);
        console.log("test float " + f);
    });
};