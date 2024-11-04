// wasm interop code

export function alert(msg) {
    window.alert(msg);
}

export function boardDetails(wid, heigh) {
    window.exosphere.board.width = wid;
    window.exosphere.board.height = heigh;
}

export function register_listeners(on_tick_wasm, on_key_wasm, on_mouse_wasm, on_ws_wasm) {
    // set the event listener that kicks off the core of the program
    let overlay = setup_gridoverlay_renderer();
    const mainloop = () => {
        overlay(window.innerWidth/2 - window.exosphere.board.offX, window.innerHeight/2 - window.exosphere.board.offY, window.exosphere.board.width, window.exosphere.board.height, window.exosphere.fabbers, window.exosphere.territories);
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
            width: 1000,
            height: 1000,
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