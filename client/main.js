window.onload = () => {
    var connection = new Connection("ws://localhost:3000/game");
    var protocol = connection.load_protocol(OUTGOING_PROTOCOL);
    connection.onclose = () => {
        alert("connection broken.");
    };
    connection.onopen = () => {
        protocol.Test("hello, world", 0, 12.345);
    };
    connection.onMessage("Test", (s, n, f) => {
        console.log("got test string " + s);
        console.log("test u16 " + n);
        console.log("test float " + f);
    });
};