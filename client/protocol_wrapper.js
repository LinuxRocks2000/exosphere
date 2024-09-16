/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// manages a websocket connection and wraps the parse_frame (protocolDecode and protocolEncode) functions in a call/callback pattern

class Connection {
    constructor(url) {
        this.socket = new WebSocket(url);
        this.onopen = undefined;
        this.onclose = undefined;
        this.frameEvents = {};
        this.socket.addEventListener("open", () => {
            if (this.onopen) {
                this.onopen();
            }
        });
        this.socket.addEventListener("close", () => {
            if (this.onclose) {
                this.onclose();
            }
        });
        this.socket.addEventListener("error", () => {
            if (this.onclose) {
                this.onclose();
            }
        });
        this.socket.addEventListener("message", async (msg) => {
            let bytes = await msg.data.arrayBuffer();
            let frame = protocolDecode(bytes);
            this.frameEvents[frame.shift()](...frame);
        });
    }

    onMessage(type, callback) {
        this.frameEvents[type] = callback;
    }

    load_protocol(protocol) {
        // create a send/receive handle from a protocol descriptor (see the spec in parse_frame.js)
        var ret = {};
        protocol.forEach(el => {
            ret[el.name] = (...args) => {
                this.socket.send(protocolEncode([el.name, ...args]));
            } 
        });
        return ret;
    }
}