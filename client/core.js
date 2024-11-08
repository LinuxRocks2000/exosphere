/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// wasm interop code

export function alert(msg) {
    window.alert(msg);
}

function toMouseX(x) {
    return x - window.innerWidth/2 + window.exosphere.board.offX;
}

function toMouseY(y) {
    return y - window.innerHeight/2 + window.exosphere.board.offY;
}

export function register_listeners(on_tick_wasm, on_key_wasm, on_mouse_wasm, on_ws_wasm) {
    // set the event listener that kicks off the core of the program
    let overlay = setup_gridoverlay_renderer();
    const mainloop = () => {
        overlay(window.innerWidth/2 - window.exosphere.board.offX, window.innerHeight/2 + window.exosphere.board.offY, window.exosphere.board.width, window.exosphere.board.height, window.exosphere.fabbers, window.exosphere.territories);
        exosphere.ctx.fillStyle = "rgba(0, 0, 0, 0)";
        exosphere.ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);
        exosphere.ctx.translate(window.innerWidth/2 - window.exosphere.board.offX, window.innerHeight / 2 - window.exosphere.board.offY);
        exosphere.ctx.strokeStyle = "#FFFFFF";
        exosphere.ctx.lineWidth = 2;
        exosphere.ctx.strokeRect(0, 0, exosphere.board.width, exosphere.board.height);
        exosphere.ctx.translate(-window.innerWidth/2 + window.exosphere.board.offX, -window.innerHeight / 2 + window.exosphere.board.offY);
        on_tick_wasm();
        requestAnimationFrame(mainloop);
    };
    window.exosphere = {
        board : {
            width: 0,
            height: 0,
            offX: 0,
            offY: 0
        },
        canvas : document.getElementById("game"),
        ctx : document.getElementById("game").getContext("2d"),
        territories: [],
        fabbers: []
    };
    document.getElementById("play").onclick = () => {
        let websocket = new WebSocket(document.getElementById("server").value);
        websocket.onopen = () => {
            mainloop();
            document.getElementById("gameui").style.display = "";
            document.getElementById("loginmenu").style.display = "none";
        };
        websocket.onerror = () => {
            alert("connection error");
            window.location.reload();
        };
        websocket.onclose = () => {
            alert("disconnected");
            window.location.reload();
        };
        websocket.onmessage = async (msg) => {
            let bytes = new Uint8Array(await msg.data.arrayBuffer());
            on_ws_wasm(bytes);
        };
        window.exosphere.websocket = websocket;
    };
    window.addEventListener("keydown", evt => {
        on_key_wasm(evt.key, 1);
    });
    window.addEventListener("keyup", evt => {
        on_key_wasm(evt.key, 0);
    });
    window.addEventListener("pointermove", evt => {
        on_mouse_wasm(toMouseX(evt.clientX), toMouseY(evt.clientY), 0);
    });
    window.addEventListener("pointerup", evt => {
        on_mouse_wasm(toMouseX(evt.clientX), toMouseY(evt.clientY), 1);
    });
    window.addEventListener("pointerdown", evt => {
        on_mouse_wasm(toMouseX(evt.clientX), toMouseY(evt.clientY), 2);
    });
    function onresize() {
        window.exosphere.canvas.width = window.innerWidth;
        window.exosphere.canvas.height = window.innerHeight;
        document.getElementById("grid-overlay").width = window.innerWidth;
        document.getElementById("grid-overlay").height = window.innerHeight;
    }
    
    onresize();
    window.addEventListener("resize", onresize);
}

export function send_ws(data) {
    window.exosphere.websocket.send(data);
}

export function get_input_value(id) {
    return document.getElementById(id).value;
}

export function set_board_size(w, h) {
    window.exosphere.board.width = w;
    window.exosphere.board.height = h;
}

export function set_offset(x, y) {
    window.exosphere.board.offX = x;
    window.exosphere.board.offY = y;
}

export function set_time(tick, stage, phase) {
    document.getElementById("curtime").innerText = tick;
    document.getElementById("stagetime").innerText = stage;
    document.getElementById("phase").innerText = phase;
    let tb = document.getElementById("timebar");
    if (stage - tick < 75) {
        tb.style.backgroundColor = "red";
    }
    else if (stage - tick < 150) {
        tb.style.backgroundColor = "yellow";
    }
    else {
        tb.style.backgroundColor = "green";
    }
}