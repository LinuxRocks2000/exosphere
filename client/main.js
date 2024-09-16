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

// why yes, this code *is* a mess, thanks for noticing
// this is mostly thrown together to test the server
// there will be overhauls in the future

/*
    TODO:
    There is a problem with angle where, when n_a is near 0 and o_a is near 360, the interpolation steps
    perform a full rotation around the circle instead of a small rotation between 360 and 0.
    this is the use case for loopification, but I don't feel like implementing that right now, so future me gets to solve it! he's welcome!
*/

const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

const background_overlay = setup_gridoverlay_renderer();

const VERSION = 0; // bump for possibly-breaking protocol or gameplay changes

const TEST = ["EXOSPHERE", 128, 4096, 115600, 123456789012345n, -64, -4096, -115600, -123456789012345n, -4096.51220703125, -8192.756, VERSION];

const MS_PER_FRAME = 1000 / 30;

var viewX = 0; // in world units, correspond to the center of the screen
var viewY = 0;

var rawMX = 0; // raw mouse x
var rawMY = 0; // raw mouse y

var keysDown = {};
var mouseDown = false;

var went_down_on = [undefined, undefined]; // (piece, index) moving and deleting strategy nodes can be done at any time, as can line insertion. 
// extending strategy paths at the end requires selection, as does clearing strategy nodes.

var mouseX = 0; // in world units
var mouseY = 0;

var hovered = undefined; // current hovered thing
var selected = undefined; // current selected thing

var should_place_node = true; // should we place a strategy node when the pointer goes up? usually true, sets to false when you use a gesture (like dragging a point)

var m_id = undefined;
var money = 0;

var lastFrameTime = 0;

var gameboardWidth = 0;
var gameboardHeight = 0;

var is_playing = false;
var is_strategy = false;
var is_io = false;
var time_in_stage = 0;
var time_so_far = 0;

var has_placed_castle = false;

var can_place_here = true; // for CASTLES ONLY! todo: rename
// this code really needs a refactor :sigh:

var slot = 0;

var clients = {}; // managed list of clients in the room

function onresize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    document.getElementById("grid-overlay").width = window.innerWidth;
    document.getElementById("grid-overlay").height = window.innerHeight;
}

onresize();

var res_cache = {};
function getRes(res) {
    if (!res_cache[res]) {
        res_cache[res] = document.getElementById(res);
    }
    return res_cache[res];
}

var active_territories = {};
var active_fabbers = {};
var pieces = {}; // hash table of ids
// the relationship between Bevy indexes and memory layout is unpredictable, so it makes more sense to use a hash table than a potentially extremely oversized padded array

window.addEventListener("resize", onresize);

function dot(v1, v2) {
    return v1[0] * v2[0] + v1[1] * v2[1];
}

function loopify(one, two) {
    let delta = two - one;
    if (delta > Math.PI) {
        return delta - 2 * Math.PI;
    }
    if (delta < -Math.PI) {
        return delta + 2 * Math.PI;
    }
    return delta;
}

function isFriendly(player) {
    if (player == 0) {
        return false;
    }
    if (!clients[player]) {
        return false;
    }
    if (clients[player].id == m_id) {
        return true;
    }
    if (clients[player].slot == slot && slot >= 2) {
        return true;
    }
    return false;
}

function mainloop() {
    background_overlay(window.innerWidth / 2 - viewX, window.innerHeight / 2 + viewY, gameboardWidth, gameboardHeight, pieces, active_fabbers, active_territories);
    mouseX = viewX + rawMX - window.innerWidth / 2;
    mouseY = viewY + rawMY - window.innerHeight / 2;
    if (mouseX < 0) {
        mouseX = 0;
    }
    if (mouseY < 0) {
        mouseY = 0;
    }
    if (mouseX > gameboardWidth) {
        mouseX = gameboardWidth;
    }
    if (mouseY > gameboardHeight) {
        mouseY = gameboardHeight;
    }
    var translateX = window.innerWidth / 2 - viewX; // in screen units, adjust the view so it is in fact centered on (viewX, viewY)
    var translateY = window.innerHeight / 2 - viewY;
    ctx.fillStyle = "rgba(0, 0, 0, 0)";
    ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);
    //ctx.drawImage(document.getElementById("grid-overlay"), 0, 0);//ctx.putImageData(imageData, 0, 0);
    ctx.fillStyle = "white";
    ctx.font = "10px sans-serif";
    ctx.textAlign = "right";
    ctx.fillText(time_so_far + "/" + time_in_stage + " in " + (is_playing ? (is_strategy ? "strategy" : "play") : "wait") + "mode.", window.innerWidth - 30, 30);
    ctx.fillText("$" + money, window.innerWidth - 30, 60);
    ctx.textAlign = "left";
    ctx.translate(translateX, translateY);
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, gameboardWidth, gameboardHeight);
    var delta = window.performance.now() - lastFrameTime;
    delta /= MS_PER_FRAME;
    var inv_delta = 1 - delta;

    hovered = undefined;
    for (item of Object.values(pieces)) {
        var fString = isFriendly(item.owner) ? "friendly" : "enemy";
        var x = item.x_n * delta + item.x_o * inv_delta;
        var y = item.y_n * delta + item.y_o * inv_delta;
        var a = item.a_o + loopify(item.a_o, item.a_n) * delta;
        ctx.translate(x, y);
        ctx.rotate(a);
        if (item.type == 0) {
            ctx.drawImage(getRes("basic_fighter_" + fString), -41/2, -41/2);
        }
        else if (item.type == 1) {
            ctx.drawImage(getRes("castle_" + fString), -30, -30);
        }
        else if (item.type == 2) {
            ctx.fillStyle = "white";
            ctx.fillRect(-2.5, -2.5, 5, 5);
        }
        else if (item.type == 3) {
            ctx.drawImage(getRes("tie_fighter_" + fString), -20, -25);
        }
        ctx.rotate(-a);
        ctx.translate(-x, -y);
        var m_dx = mouseX - x;
        var m_dy = mouseY - y;
        var d = m_dx * m_dx + m_dy * m_dy;
        if (canUpdateStrategy(item)) {
            var lastPos = [x, y];
            item.strategy.forEach(strat => {
                // each strategy entry is either a vec2 [x, y] or a gamepiece. If a gamepiece, this object is
                // a) in seeker mode
                // b) going to travel through a teleportal
                // the important semantics are in how the user interacts with it (you can't *move* the strategy post if it's on a gamepiece)
                // and how the game engines simulate it (ships won't shoot while on direct approach to a teleportal, lest they destroy it and ruin their route)
                // other gamepieces must be the end of a ship route *unless* they are a teleportal
                if (Array.isArray(strat)) {
                    ctx.beginPath();
                    ctx.lineWidth = 1;
                    ctx.strokeStyle = "blue";
                    ctx.moveTo(lastPos[0], lastPos[1]);
                    ctx.lineTo(strat[0], strat[1]);
                    ctx.stroke();
                    ctx.beginPath();
                    ctx.fillStyle = "blue";
                    ctx.arc(strat[0], strat[1], 3, 0, Math.PI * 2);
                    ctx.fill();
                    lastPos[0] = strat[0];
                    lastPos[1] = strat[1];
                }
            });
            if (item.strategy_endcap) {
                var tx = 0;
                var ty = 0;
                if (item.strategy.length > 0) {
                    tx = item.strategy[item.strategy.length - 1][0];
                    ty = item.strategy[item.strategy.length - 1][1];
                }
                else {
                    tx = item.x_n;
                    ty = item.y_n;
                }
                ctx.translate(tx, ty);
                ctx.rotate(item.strategy_endcap);
                ctx.lineWidth = 5;
                ctx.strokeStyle = "white";
                ctx.beginPath();
                ctx.moveTo(20, 20);
                ctx.lineTo(30, 0);
                ctx.lineTo(20, -20);
                ctx.stroke();
                ctx.rotate(-item.strategy_endcap);
                ctx.translate(-tx, -ty);
            }
            if (d < 15 * 15) {
                ctx.strokeStyle = "#FFFFFF";
                ctx.lineWidth = 2;
                ctx.beginPath();
                ctx.arc(mouseX, mouseY, 8, 0, Math.PI * 2);
                ctx.stroke();
                hovered = item;
            }
            if (selected == item) {
                ctx.strokeStyle = "rgb(0, 190, 255)";
                ctx.lineWidth = 3;
                ctx.strokeRect(x - 40, y - 40, 80, 80);
            }
        }
        if (d < 20 * 20 && item.owner != 0) {
            ctx.fillStyle = isFriendly(item.owner) ? "rgb(0, 190, 255)" : "rgb(255, 88, 0)";
            ctx.font = "12px sans-serif";
            var width = ctx.measureText(clients[item.owner].banner).width;
            ctx.beginPath();
            ctx.moveTo(x - width - 5 - 35, y - 6 - 2.5);
            ctx.lineTo(x - 2.5 - 35, y - 6 - 2.5);
            ctx.lineTo(x - 2.5 - 35, y - 3);
            ctx.lineTo(x - 2.5 - 30, y);
            ctx.lineTo(x - 2.5 - 35, y + 3);
            ctx.lineTo(x - 2.5 - 35, y + 6 + 2.5);
            ctx.lineTo(x - width - 5 - 35, y + 6 + 2.5);
            ctx.closePath();
            ctx.fill();
            ctx.fillStyle = "black";
            ctx.fillText(clients[item.owner].banner, x - width - 4 - 35, y + 2.5);
        }
    }
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(mouseX, mouseY, 10, 0, Math.PI * 2);
    ctx.stroke();
    if (!has_placed_castle) {
        can_place_here = true;
        for (tid of Object.keys(active_territories)) {
            let dx = mouseX - pieces[tid].x_n;
            let dy = mouseY - pieces[tid].y_n;
            let d = Math.sqrt(dx * dx + dy * dy);
            if (d < active_territories[tid] + 600.0) {
                can_place_here = false;
            }
        }
        ctx.beginPath();
        ctx.strokeStyle = can_place_here ? "#AAAAAA" : "#AA0000";
        ctx.lineWidth = 1;
        ctx.arc(mouseX, mouseY, 600, 0, Math.PI * 2);
        ctx.stroke();
    }

    ctx.translate(-translateX, -translateY);
    requestAnimationFrame(mainloop);
}

function canUpdateStrategy(obj) {
    // return true if we have the actual ability to update the strategy for a given object
    if (obj.owner != m_id) {
        return false; // we can never move an object that isn't ours
    }
    return [0, 3].indexOf(obj.type) != -1;
}

function play() {
    var connection = new Connection(document.getElementById("server").value);
    var protocol = connection.load_protocol(OUTGOING_PROTOCOL);


    canvas.addEventListener("pointerup", () => {
        mouseDown = false;
        if (slot == 0) { // past this point, spectators can't do anything
            return;
        }
        if (has_placed_castle) {
            let did_place = false;
            if (should_place_node && is_playing && is_strategy) { // do insert checks
                var nearest_projection = [];
                var nearest_index = 0;
                var nearest_val = Infinity;
                var nearest_piece = undefined;
                Object.values(pieces).forEach(piece => {
                    var last_vec = [piece.x_n, piece.y_n];
                    piece.strategy.forEach((node, i) => {
                        var vec = [0, 0];
                        if (Array.isArray(node)) {
                            vec[0] = node[0];
                            vec[1] = node[1];
                        }
                        var dx_line = last_vec[0] - vec[0];
                        var dy_line = last_vec[1] - vec[1];
                        var proj = [0, 0];
                        var len = dx_line * dx_line + dy_line * dy_line;
                        if (len == 0) {
                            proj[0] = last_vec[0];
                            proj[1] = last_vec[1];
                        }
                        else {
                            var coeff = dot([mouseX - last_vec[0], mouseY - last_vec[1]], [vec[0] - last_vec[0], vec[1] - last_vec[1]]) / len;
                            if (coeff < 0) {
                                coeff = 0;
                            }
                            if (coeff > 1) {
                                coeff = 1;
                            }
                            var l = [vec[0] - last_vec[0], vec[1] - last_vec[1]];
                            proj[0] = last_vec[0] + coeff * l[0];
                            proj[1] = last_vec[1] + coeff * l[1];
                        }
                        var dx = proj[0] - mouseX;
                        var dy = proj[1] - mouseY;
                        if (dx * dx + dy * dy < nearest_val) {
                            nearest_val = dx * dx + dy * dy;
                            nearest_projection = proj;
                            nearest_index = i;
                            nearest_piece = piece;
                        }
                        last_vec = vec;
                    });
                });
                if (nearest_val < 10 * 10) {
                    nearest_piece.strategy.splice(nearest_index, 0, nearest_projection);
                    protocol.StrategyPointAdd(nearest_piece.id, nearest_index, nearest_projection[0], nearest_projection[1]);
                    did_place = true;
                }
            }
            var pointHovered_i = 0;
            var hovered_piece = undefined;
            Object.values(pieces).forEach(piece => {
                piece.strategy.forEach((node, i) => {
                    if (Array.isArray(node)) {
                        var dx = node[0] - mouseX;
                        var dy = node[1] - mouseY;
                        var d = dx * dx + dy * dy;
                        if (d < 4 * 4) {
                            pointHovered_i = i;
                            hovered_piece = piece;
                        }
                    }
                });
            });
            if (keysDown["d"]) {
                hovered_piece.strategy.splice(pointHovered_i, 1);
                protocol.StrategyRemove(hovered_piece.id, pointHovered_i);
            }
            if (!did_place) {
                if (hovered) {
                    selected = hovered;
                }
                else if (selected && is_playing && is_strategy) {
                    if (should_place_node) {
                        selected.strategy.push([mouseX, mouseY]);
                        protocol.StrategyPointAdd(selected.id, selected.strategy.length - 1, mouseX, mouseY);
                    }
                }
            }
            if (selected == undefined && is_playing && is_strategy) { // if there's no piece selected, and we're actually playing, and 
                let placer = document.querySelector("#buyshipmenu input:checked + label");
                if (placer) {
                    if (money >= placer.getAttribute("--data-price")) {
                        protocol.PlacePiece(mouseX, mouseY, placer.getAttribute("--data-place-type"));
                        if (!keysDown[" "]) {
                            placer.previousElementSibling.checked = false; // uncheck it so we don't end up placing an extra piece
                            // if the user is holding down spacebar, we don't clear it, because that's so damn annoying
                        }
                    }
                }
            }
        }
        else if (is_io || !is_playing) {
            if (can_place_here) {
                protocol.PlacePiece(mouseX, mouseY, 1);
                has_placed_castle = true;
            }
        }
        should_place_node = true;
        went_down_on[0] = undefined;
        went_down_on[1] = undefined;
    });

    canvas.addEventListener("pointerdown", () => {
        mouseDown = true;
        if (is_playing && is_strategy) {
            Object.values(pieces).forEach((piece, obj_i) => {
                piece.strategy.forEach((node, i) => {
                    if (Array.isArray(node)) {
                        var dx = node[0] - mouseX;
                        var dy = node[1] - mouseY;
                        var d = dx * dx + dy * dy;
                        if (d < 6 * 6) {
                            went_down_on[0] = piece.id;
                            went_down_on[1] = i;
                            should_place_node = false;
                        }
                    }
                });
            });
        }
    });
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
    connection.onMessage("Metadata", (id, width, height, s) => {
        slot = s;
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
            y_n: y,
            a_n: a,
            owner : owner,
            type: type,
            id: id,
            strategy_endcap: undefined,
            strategy: [[x, y]]
        }
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
        }/*
        for (item of Object.values(pieces)) {
            item.x_o = item.x_n;
            item.y_o = item.y_n;
            item.a_o = item.a_n;
        }*/
    });
    connection.onMessage("DeleteObject", (id) => {
        delete pieces[id];
    });
    connection.onMessage("StrategyCompletion", (id, a) => {
        pieces[id].strategy.splice(0, 1);
        if (pieces[id].strategy.length != a) {
            alert("WARNING: MISMATCHED STRATEGY PATHS!");
        }
    });
    connection.onMessage("PlayerData", (id, banner, slot) => {
        clients[id] = {
            id: id,
            banner: banner,
            slot: slot
        };
    });
    connection.onMessage("YouLose", () => {
        alert("game over! you lose!");
    });
    connection.onMessage("Winner", (id) => {
        if (id == m_id) {
            alert("game over! you win!");
        }
        else {
            alert(clients[id].banner + " wins!");
        }
    });
    connection.onMessage("Territory", (id, radius) => {
        active_territories[id] = radius;
    });
    connection.onMessage("Fabber", (id, radius) => {
        active_fabbers[id] = radius;
    });
    connection.onMessage("Disconnect", () => {
        location.reload();
    });
    connection.onMessage("Money", (id, amt) => {
        clients[id].money = amt;
        if (id == m_id) {
            money = amt;
        }
    });
    document.getElementById("loginmenu").style.display = "none";
    document.getElementById("waitscreen").style.display = "";

    window.addEventListener("keyup", evt => {
        if (evt.key == "Escape") {
            selected = undefined;
        }
        if (evt.key == "c") {
            if (selected && is_playing && is_strategy) {
                selected.strategy = [];
                protocol.StrategyClear(selected.id);
            }
        }
        keysDown[evt.key] = false;
    });

    window.addEventListener("keydown", evt => {
        keysDown[evt.key] = true;
    });

    canvas.addEventListener("pointermove", evt => {
        rawMX = evt.clientX;
        rawMY = evt.clientY;
        if (is_playing && is_strategy) {
            if (mouseDown && went_down_on[0]) {
                if (Array.isArray(pieces[went_down_on[0]].strategy[went_down_on[1]])) {
                    pieces[went_down_on[0]].strategy[went_down_on[1]][0] = mouseX;
                    pieces[went_down_on[0]].strategy[went_down_on[1]][1] = mouseY;
                    protocol.StrategyPointUpdate(went_down_on[0], went_down_on[1], mouseX, mouseY); // todo: make this suck less [right now we send position updates every mousemove event!]
                }
            }
            if (keysDown['r'] && selected) {
                var dx = mouseX - selected.strategy[selected.strategy.length - 1][0];
                var dy = mouseY - selected.strategy[selected.strategy.length - 1][1];
                selected.strategy_endcap = Math.atan2(dy, dx);
                protocol.StrategySetEndcapRotation(selected.id, selected.strategy_endcap); // todo: make this suck less (see above)
            }
        }
    });
}

canvas.addEventListener("wheel", evt => {
    viewX += evt.deltaX;
    viewY += evt.deltaY;
    evt.preventDefault();
    return false;
}, { passive: false });


function saveinputstate(el) {
    localStorage["inputstate_" + el.id] = el.value;
}

for (el of document.getElementsByTagName("input")) {
    if (localStorage["inputstate_" + el.id]) {
        el.value = localStorage["inputstate_" + el.id];
    }
}

for (let el of document.querySelectorAll("#buyshipmenu > div > div > label")) {
    let inp = el.previousElementSibling;
    el.addEventListener("mouseup", () => {
        if (inp.checked) {
            setTimeout(() => {
                inp.checked = false;
            }, 0);
        }
    });
}