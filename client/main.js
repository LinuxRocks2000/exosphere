/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

/* COLORS
    Friendly:(0, 190, 255)
    Enemy: (255, 88, 0)
*/

const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

const VERSION = 0; // bump for possibly-breaking protocol or gameplay changes

const TEST = ["EXOSPHERE", 128, 4096, 115600, 123456789012345n, -64, -4096, -115600, -123456789012345n, -4096.51220703125, -8192.756, VERSION];

const MS_PER_FRAME = 1000 / 30;

var viewX = 0; // in world units, correspond to the center of the screen
var viewY = 0;

var m_id = undefined;

var lastFrameTime = 0;

var gameboardWidth = 0;
var gameboardHeight = 0;

var is_playing = false;
var is_strategy = false;
var is_io = false;
var time_in_stage = 0;
var time_so_far = 0;

function onresize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

onresize();

var res_cache = {};
function getRes(res) {
    if (!res_cache[res]) {
        res_cache[res] = document.getElementById(res);
    }
    return res_cache[res];
}

var pieces = {}; // hash table of ids
// the relationship between Bevy indexes and memory layout is unpredictable, so it makes more sense to use a hash table than a potentially extremely oversized padded array

window.addEventListener("resize", onresize);

function mainloop() {
    var translateX = window.innerWidth / 2 - viewX; // in screen units, adjust the view so it is in fact centered on (viewX, viewY)
    var translateY = window.innerHeight / 2 - viewY;
    ctx.fillStyle = "#000022";
    ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
    ctx.fillStyle = "white";
    ctx.fillText(time_so_far + "/" + time_in_stage + " in " + (is_playing ? (is_strategy ? "strategy" : "play") : "wait") + "mode.", 30, 30);
    ctx.translate(translateX, translateY);
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, gameboardWidth, gameboardHeight);

    var delta = window.performance.now() - lastFrameTime;
    console.log(delta);
    delta /= MS_PER_FRAME;
    var inv_delta = 1 - delta;

    for (item of Object.values(pieces)) {
        var fString = item.owner == m_id ? "friendly" : "enemy";
        if (item.type == 0) {
            ctx.drawImage(getRes("basic_fighter_" + fString), item.x_n * delta + item.x_o * inv_delta, item.y_n * delta + item.y_o * inv_delta);
        }
    }

    ctx.translate(-translateX, -translateY);
    requestAnimationFrame(mainloop);
}

function play() {
    var connection = new Connection(document.getElementById("server").value);
    var protocol = connection.load_protocol(OUTGOING_PROTOCOL);
    connection.onclose = () => {
        alert("connection zonked.");
    };
    connection.onopen = () => {
    };
    connection.onMessage("Test", (...args) => {
        var passed = true;
        args.forEach((item, i) => {
            if (item != TEST[i]) {
                passed = false;
                console.log(item + " (" + i + ") is not equal to " + TEST[i]);
            }
        });
        if (passed) {
            protocol.Test(...args);
            protocol.Connect(document.getElementById("nickname").value, "");
        }
        else {
            if (confirm("Server failed initial test. This may be because the client is out of date. Proceed anyways?")) {
                protocol.Test(...args);
            }
            else {
                alert("session aborted. kill yourself.");
            }
        }
    });
    connection.onMessage("Metadata", (id, width, height) => {
        m_id = id;
        gameboardWidth = width;
        gameboardHeight = height;
        document.getElementById("waitscreen").style.display = "none";
        document.getElementById("gameui").style.display = "";
        mainloop();
    });
    connection.onMessage("ObjectCreate", (x, y, a, owner, id, type) => {
        pieces[id] = {
            x_o: x, // last frame
            y_o: y,
            a_o: a,
            x_n: x, // current frame
            y_n: x,
            a_n: a,
            owner : owner,
            type: type,
            id: id
        }
        console.log(`new object at ${x},${y} id ${id} type ${type}`);
    });
    connection.onMessage("ObjectMove", (id, x, y, a) => {
        let p = pieces[id];
        if (p) {
            p.x_o = p.x_n;
            p.y_o = p.y_n;
            p.a_o = p.a_n;
            p.x_n = x;
            p.y_n = y;
            p.a_n = a;
        }
    });
    connection.onMessage("ObjectTrajectoryUpdate", (id, x, y, a, xv, yv, av) => {
        let p = pieces[id];
        if (p) {
            p.x_o = x;
            p.y_o = y;
            p.a_o = a;
            p.x_n = x + xv;
            p.y_n = y + yv;
            p.a_n = a + av;
        }
    });
    connection.onMessage("GameState", (byte, tick, totalTime) => {
        lastFrameTime = window.performance.now();
        time_in_stage = totalTime;
        time_so_far = tick;
        if (byte & 128) {
            is_io = true;
        }
        else {
            is_io = false;
        }
        if (byte & 64) {
            is_playing = true;
        }
        else {
            is_playing = false;
        }
        if (byte & 32) {
            is_strategy = true;
        }
        else {
            is_strategy = false;
        }
    });
    document.getElementById("loginmenu").style.display = "none";
    document.getElementById("waitscreen").style.display = "";
}

window.addEventListener("wheel", evt => {
    viewX += evt.deltaX;
    viewY += evt.deltaY;
});