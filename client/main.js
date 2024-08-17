/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

const VERSION = 0; // bump for possibly-breaking protocol or gameplay changes

const TEST = ["EXOSPHERE", 128, 4096, 115600, 123456789012345n, -64, -4096, -115600, -123456789012345n, -4096.51220703125, -8192.756, VERSION];

var viewX = 0;
var viewY = 0;

var gameboardWidth = 0;
var gameboardHeight = 0;

function onresize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

onresize();

window.addEventListener("resize", onresize);

function mainloop() {
    ctx.fillStyle = "#000022";
    ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
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
    connection.onMessage("Metadata", (width, height) => {
        gameboardWidth = width;
        gameboardHeight = height;
        document.getElementById("waitscreen").style.display = "none";
        document.getElementById("gameui").style.display = "";
    });
    document.getElementById("loginmenu").style.display = "none";
    document.getElementById("waitscreen").style.display = "";
    mainloop();
}