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
            let bytes = await msg.data.bytes();
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