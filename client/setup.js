// set up the listeners

function toMouseX(x) {
  return Math.min(
    Math.max(x - window.innerWidth / 2 + window.exosphere.board.offX, 0),
    window.exosphere.board.width,
  );
}

function toMouseY(y) {
  return Math.min(
    Math.max(y - window.innerHeight / 2 + window.exosphere.board.offY, 0),
    window.exosphere.board.height,
  );
}

function setup(state) {
  // set the event listener that kicks off the core of the program
  window.exosphere = {
    board: {
      width: 0,
      height: 0,
      offX: 0,
      offY: 0,
      rawMy: 0,
      rawMx: 0,
    },
    overlay: setup_gridoverlay_renderer(),
    state: state,
    canvas: document.getElementById("game"),
    ctx: document.getElementById("game").getContext("2d"),
  };
  const mainloop = () => {
    exosphere.ctx.fillStyle = "rgba(0, 0, 0, 0)";
    exosphere.ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);
    let offX = window.innerWidth / 2 - window.exosphere.board.offX;
    let offY = window.innerHeight / 2 - window.exosphere.board.offY;
    exosphere.ctx.translate(offX, offY);
    exosphere.ctx.strokeStyle = "#FFFFFF";
    exosphere.ctx.lineWidth = 2;
    exosphere.ctx.strokeRect(
      0,
      0,
      exosphere.board.width,
      exosphere.board.height,
    );
    state.set_mouse_pos(
      toMouseX(window.exosphere.board.rawMx),
      toMouseY(window.exosphere.board.rawMy),
    );
    state.tick();
    exosphere.ctx.translate(-offX, -offY);
    requestAnimationFrame(mainloop);
  };
  document.getElementById("play").onclick = () => {
    let websocket = new WebSocket(document.getElementById("server").innerHTML);
    websocket.onopen = () => {
      mainloop();
      document.getElementById("gameui").style.display = "";
      document.getElementById("loginmenu").style.display = "none";
    };
    websocket.onerror = () => {
      alert("connection error");
      window.location.reload();
    };
    websocket.onmessage = async (msg) => {
      let bytes = new Uint8Array(await msg.data.arrayBuffer());
      window.exosphere.state.on_message(bytes);
    };
    window.exosphere.websocket = websocket;
    window.addEventListener("keydown", (evt) => {
      window.exosphere.state.key_down(evt.key);
    });
    window.addEventListener("keyup", (evt) => {
      window.exosphere.state.key_up(evt.key);
    });
    window.addEventListener("pointermove", (evt) => {
      window.exosphere.board.rawMx = evt.clientX;
      window.exosphere.board.rawMy = evt.clientY;
    });
    window.exosphere.canvas.addEventListener("pointerup", () => {
      window.exosphere.state.mouse_up();
    });
    window.exosphere.canvas.addEventListener("pointerdown", () => {
      window.exosphere.state.mouse_down();
    });
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
