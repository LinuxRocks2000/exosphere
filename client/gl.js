/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// gpu-assisted stuff
// grid overlay mostly
// renders the grid overlay into the grid-overlay canvas with webgl on demand

function setup_gridoverlay_renderer() { // this is a higher-order function that returns a function that draws the background to the grid-overlay canvas
    let canvas = document.getElementById("grid-overlay", {preserveDrawingBuffer : true});
    let webgl = canvas.getContext("webgl2");
    var program = webgl.createProgram();
    function compile_shader(src, type) {
        let shader = webgl.createShader(type);
        webgl.shaderSource(shader, src);
        webgl.compileShader(shader);
        let status = webgl.getShaderParameter(shader, webgl.COMPILE_STATUS);
        if (!status) {
            var compilationLog = webgl.getShaderInfoLog(shader);
            console.log('Shader compiler log: ' + compilationLog);
            alert("Error: GL initialization failed. You may want to disable GPU-assisted rendering. Developer information: " + status);
            return undefined;
        }
        return shader;
    }
    function compile_shader_embedded(id) {
        let embedded_shader = document.getElementById(id);
        if (embedded_shader.type == "shader/fragment") {
            return compile_shader(embedded_shader.innerHTML, webgl.FRAGMENT_SHADER);
        }
        else if (embedded_shader.type == "shader/vertex") {
            return compile_shader(embedded_shader.innerHTML, webgl.VERTEX_SHADER);
        }
        alert("Error: invalid shader at " + id + ". You may want to disable GPU-assisted rendering. Developer information: " + embedded_shader.type);
        return undefined;
    }
    function get_attrib_location(name) {
        let location = webgl.getAttribLocation(program, name);
        if (location == -1) {
            alert("Error: Can't find attribute location. You may want to disable GPU-assisted rendering. Developer information: " + name);
            return undefined;
        }
        return location;
    }
    function get_uniform_location(name) {
        let location = webgl.getUniformLocation(program, name);
        if (location == -1) {
            alert("Error: Can't find uniform location. You may want to disable GPU-assisted rendering. Developer information: " + name);
            return undefined;
        }
        return location;
    }
    let vertex = compile_shader_embedded("vertex-shader");
    let fragment = compile_shader_embedded("fragment-shader");
    webgl.attachShader(program, vertex);
    webgl.attachShader(program, fragment);
    webgl.linkProgram(program);
    webgl.useProgram(program);

    let texture = new Float32Array([ // shamelessly copypasted
        -1.0, 1.0, // top left
        -1.0, -1.0, // bottom left
        1.0, 1.0, // top right
        1.0, -1.0, // bottom right
    ]);

    //let boarddata_handle = get_uniform_location("boarddata");

    let texture_buf = webgl.createBuffer();
    webgl.bindBuffer(webgl.ARRAY_BUFFER, texture_buf);
    webgl.bufferData(webgl.ARRAY_BUFFER, texture, webgl.STATIC_DRAW);

    let position = get_attrib_location("position");
    webgl.enableVertexAttribArray(position);
    webgl.vertexAttribPointer(position, 2, webgl.FLOAT, webgl.FALSE, 2 * 4, 0);
    
    let boffset = get_uniform_location("boardOffset");
    let bsize = get_uniform_location("boardSize");
    let fabbers_handle = get_uniform_location("fabbers");
    let fabbers_buffer = new Float32Array(64 * 3);
    let fabber_count_handle = get_uniform_location("fabber_count");
    let territories_handle = get_uniform_location("territories");
    let territory_buffer = new Float32Array(64 * 3);
    let territory_count_handle = get_uniform_location("territory_count");

    return function (boffX, boffY, bWid, bHeigh, fabbers, territories) {
        let territory_count = 0;
        territories.forEach(terr => {
            territory_buffer[territory_count * 3] = terr.x;
            territory_buffer[territory_count * 3 + 1] = terr.y;
            territory_buffer[territory_count * 3 + 2] = terr.rad * (terr.isFriendly ? 1 : -1);
            territory_count += 1;
        });
        let fabber_count = 0;
        fabbers.forEach(fab => {
            fabbers_buffer[fabber_count * 3] = fab.x;
            fabbers_buffer[fabber_count * 3 + 1] = fab.y;
            fabbers_buffer[fabber_count * 3 + 2] = fab.rad * (fab.isFriendly ? 1 : -1);
            fabber_count += 1;
        });
        webgl.uniform1i(territory_count_handle, territory_count);
        webgl.uniform3fv(territories_handle, territory_buffer);
        webgl.uniform1i(fabber_count_handle, fabber_count);
        webgl.uniform3fv(fabbers_handle, fabbers_buffer);
        webgl.uniform2f(boffset, boffX, boffY);
        webgl.uniform2f(bsize, bWid, bHeigh);
        webgl.viewport(0, 0, window.innerWidth, window.innerHeight);
        webgl.drawArrays(webgl.TRIANGLE_STRIP, 0, 4);
    }
}