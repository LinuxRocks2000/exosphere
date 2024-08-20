/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// parse a protocol frame into a javascript object, and vice versa

/*
    each type must be in the format
    "typename": {
        encode(data, position, bytes) {
            // write data into bytes starting at position, using data as the javascript equivalent of this datatype (so number for any uX or iX or fX, strings for String)
        },
        decode(position, bytes) { 
            // decode from bytes at position and return a javascript equivalent of this datatype (must be feedable back into encode)
        },
        size(me) {
            // return the encoded size of an object
        }

        // bytes should always be a DataView of a Uint8Array
    }
*/

const PROTOCOL_TYPES = {
    'String': {
        encode(data, position, bytes) {
            var view = new TextEncoder().encode(data);
            bytes.setUint16(position, view.byteLength, true);
            for (var x = 0; x < view.length; x++) {
                bytes.setUint8(position + 2 + x, view[x]);
            }
        },
        decode(position, bytes) {
            var len = bytes.getUint16(position, true);
            var buffer = new Uint8Array(len);
            for (var x = 0; x < len; x ++) {
                buffer[x] = bytes.getUint8(position + 2 + x);
            }
            return new TextDecoder().decode(buffer);
        },
        size(me) {
            return 2 + new TextEncoder().encode(me).length; // todo: make this more efficient
        }
    },
    'u8': {
        encode(data, position, bytes) {
            bytes.setUint8(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getUint8(position, true);
        },
        size() {
            return 1;
        }  
    },
    'u16': {
        encode(data, position, bytes) {
            bytes.setUint16(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getUint16(position, true);
        },
        size() {
            return 2;
        }
    },
    'u32': {
        encode(data, position, bytes) {
            bytes.setUint32(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getUint32(position, true);
        },
        size() {
            return 4;
        }
    },
    'u64': {
        encode(data, position, bytes) {
            bytes.setBigUint64(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getBigUint64(position, true);
        },
        size() {
            return 8;
        }
    },
    'i8': {
        encode(data, position, bytes) {
            bytes.setInt8(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getInt8(position, true);
        },
        size() {
            return 1;
        }
    },
    'i16': {
        encode(data, position, bytes) {
            bytes.setInt16(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getInt16(position, true);
        },
        size() {
            return 2;
        }
    },
    'i32': {
        encode(data, position, bytes) {
            bytes.setInt32(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getInt32(position, true);
        },
        size() {
            return 4;
        }
    },
    'i64': {
        encode(data, position, bytes) {
            bytes.setBigInt64(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getBigInt64(position, true);
        },
        size() {
            return 8;
        }
    },
    'f32': {
        encode(data, position, bytes) {
            bytes.setFloat32(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getFloat32(position, true);
        },
        size() {
            return 4;
        }
    },
    'f64': {
        encode(data, position, bytes) {
            bytes.setFloat64(position, data, true);
        },
        decode(position, bytes) {
            return bytes.getFloat64(position, true);
        },
        size() {
            return 8;
        }
    }
};

const INCOMING_PROTOCOL = [
    {
        name: "Test",
        layout: [
            'String',
            'u8',
            'u16',
            'u32',
            'u64',
            'i8',
            'i16',
            'i32',
            'i64',
            'f32',
            'f64',
            'u8'
        ]
    },
    {
        name: "GameState",
        layout: [
            'u8',
            'u16',
            'u16'
        ]
    },
    {
        name: "Metadata",
        layout: [
            'u64',
            'f32',
            'f32'
        ]
    },
    {
        name: "ObjectCreate",
        layout: [
            'f32',
            'f32',
            'f32',
            'u64',
            'u32',
            'u16'
        ]
    },
    {
        name: "ObjectMove",
        layout: [
            'u32',
            'f32',
            'f32',
            'f32'
        ]
    }
];

const OUTGOING_PROTOCOL = [
    {
        name: "Test",
        layout: [
            'String',
            'u8',
            'u16',
            'u32',
            'u64',
            'i8',
            'i16',
            'i32',
            'i64',
            'f32',
            'f64',
            'u8'
        ]
    },
    {
        name: "Connect",
        layout: [
            'String',
            'String'
        ]
    }
];

function protocolDecode(message, typeset = PROTOCOL_TYPES, protocolset = INCOMING_PROTOCOL) { // returns an array. first element is the message name (like "Test"). next elements are the message content, in order.
    var view = new DataView(message.buffer);
    var position = 1;
    var pItem = protocolset[view.getUint8(0)];
    var ret = [pItem.name];
    pItem.layout.forEach(type => {
        var data = typeset[type].decode(position, view);
        position += typeset[type].size(data); // todo: make this less bad
        ret.push(data);
    });
    return ret;
}

function protocolEncode(message, typeset = PROTOCOL_TYPES, protocolset = OUTGOING_PROTOCOL) { // takes an array formatted like the return of protocolDecode and returns an ArrayBuffer
    var pItemInd = protocolset.findIndex(p => p.name == message[0]);
    var pItem = protocolset[pItemInd];
    var size = 1;
    for (var x = 0; x < message.length - 1; x++) {
        size += typeset[pItem.layout[x]].size(message[x + 1]);
    }
    var ret = new Uint8Array(size);
    var view = new DataView(ret.buffer);
    view.setUint8(0, pItemInd);
    var position = 1;
    for (var x = 0; x < message.length - 1; x++) {
        typeset[pItem.layout[x]].encode(message[x + 1], position, view);
        position += typeset[pItem.layout[x]].size(message[x + 1]);
    }
    return ret.buffer;
}