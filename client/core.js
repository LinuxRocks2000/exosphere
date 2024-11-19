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

export function render_background(fabbers_buf, fabber_count, territories_buf, territory_count) {
    window.exosphere.overlay(window.innerWidth/2 - window.exosphere.board.offX, window.innerHeight/2 + window.exosphere.board.offY,
    window.exosphere.board.width, window.exosphere.board.height, fabbers_buf, fabber_count, territories_buf, territory_count);
}


export function ctx_draw_image(asset, x, y, a, w, h) {
    let image = document.getElementById(asset);
    if (!image) {
        image = new Image();
        image.src = "res/" + asset;
        image.id = asset;
        document.getElementById("res").appendChild(image);
    }
    window.exosphere.ctx.translate(x, y);
    window.exosphere.ctx.rotate(a);
    window.exosphere.ctx.translate(-w/2, -h/2);
    window.exosphere.ctx.drawImage(image, 0, 0);
    window.exosphere.ctx.translate(w/2, h/2);
    window.exosphere.ctx.rotate(-a);
    window.exosphere.ctx.translate(-x, -y);
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
        tb.style.backgroundColor = "rgb(97, 71, 0)";
    }
    else {
        tb.style.backgroundColor = "green";
    }
}

export function ctx_stroke(wid, color) {
    window.exosphere.ctx.lineWidth = wid;
    window.exosphere.ctx.strokeStyle = color;
}

export function ctx_fill(color) {
    window.exosphere.ctx.fillStyle = color;
}

export function ctx_outline_circle(x, y, rad) {
    window.exosphere.ctx.beginPath();
    window.exosphere.ctx.arc(x, y, rad, 0, 2 * Math.PI);
    window.exosphere.ctx.stroke();
}

export function ctx_fill_circle(x, y, rad) {
    window.exosphere.ctx.beginPath();
    window.exosphere.ctx.arc(x, y, rad, 0, 2 * Math.PI);
    window.exosphere.ctx.fill();
}

export function ctx_line_between(x1, y1, x2, y2) {
    window.exosphere.ctx.beginPath();
    window.exosphere.ctx.moveTo(x1, y1);
    window.exosphere.ctx.lineTo(x2, y2);
    window.exosphere.ctx.stroke();
}

export function set_money(m) {
    document.getElementById("money").innerText = m;
}
