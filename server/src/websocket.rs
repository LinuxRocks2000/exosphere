// a blocking, asynchronous, single-threaded websocket server!
// supports minimal http upgrade (no tls termination) and uses posix for fastliness.
use ringbuf::rb::local::LocalRb;
use ringbuf::storage::Heap;
use ringbuf::traits::Consumer;
use ringbuf::traits::{Observer, Producer};
use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::{self, TcpListener, TcpStream};
use std::os::fd::AsRawFd;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct ClientId(pub usize);

fn byte_trim(bytes: &[u8]) -> &[u8] {
    let mut start_index = 0;
    let mut end_index = bytes.len() - 1;
    while bytes[start_index].is_ascii_whitespace() {
        if start_index == bytes.len() {
            break;
        }
        start_index += 1;
    }
    while bytes[end_index].is_ascii_whitespace() {
        if end_index == start_index {
            break;
        }
        end_index -= 1;
    }
    &bytes[start_index..(end_index + 1)]
}

struct HttpUpgradeChecklist {
    connection_upgrade: bool,
    upgrade_websocket: bool, // if either of these are false, or websocket_accept is None, the request is bad and will be rejected
    websocket_accept: Option<String>,
}

impl HttpUpgradeChecklist {
    fn will_upgrade(&self) -> bool {
        if let Some(_) = self.websocket_accept {
            if self.connection_upgrade && self.upgrade_websocket {
                return true;
            }
        }
        false
    }
}

impl std::default::Default for HttpUpgradeChecklist {
    fn default() -> Self {
        Self {
            connection_upgrade: false,
            upgrade_websocket: false,
            websocket_accept: None,
        }
    }
}

struct Client {
    id: ClientId,
    socket: TcpStream,
    inbox: LocalRb<Heap<u8>>, // incoming buffer, consumed by the async eaters
    outbox: LocalRb<Heap<u8>>, // outgoing buffer, flushed occasionally by the Server
    closed: bool,
    control_state: ClientControlState,
    header_buffer: Vec<u8>,
    checklist: HttpUpgradeChecklist,
    frame: WebsocketFrameBuilder,
}

enum ClientControllerEvent<MessageType> {
    Sleep,                                  // we can't go any further without more data
    MaybeUnfinished, // we might be able to go further with more data, but don't have anything yet
    MessageReceived(ClientId, MessageType), // a websocket message was received! we may need more data to do more.
    ClientUpgraded(ClientId), // successful upgrade; more data may be needed (do not sleep)
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum ClientControlState {
    HttpFirstLine,
    HttpHeaderName,
    HttpHeaderValue,
    HttpHeaderNameStartCheckShim, // ensure that the first byte of the header name is not whitespace: if it is, we're looking at the
    // empty line that terminates the request
    WebSocketFirstTwo,
    WebSocketExtLength,
    WebSocketMasking,
    WebSocketPayload,
}

struct WebsocketFrameBuilder {
    fin: bool,                 // [1]
    rsv: u8,                   // [1..4]
    opcode: u8,                // [4..8]
    mask: bool,                // [8]
    len: u8,                   // [9..16]
    ext_len: Option<usize>,    // IF len == 126 [16..32] ELSE IF len == 127 [16..80] ELSE None
    mask_key: Option<[u8; 4]>, // IF mask [16..48] (shifted as needed by ext len) ELSE None
}

impl WebsocketFrameBuilder {
    fn length(&self) -> usize {
        match self.ext_len {
            Some(len) => len,
            None => self.len as usize,
        }
    }
}

impl std::default::Default for WebsocketFrameBuilder {
    fn default() -> Self {
        Self {
            fin: false,
            rsv: 0,
            opcode: 0,
            mask: false,
            len: 0,
            ext_len: None,
            mask_key: None,
        }
    }
}

impl Client {
    fn new_from(id: ClientId, socket: TcpStream) -> Self {
        Self {
            id,
            socket,
            inbox: LocalRb::new(4096),
            outbox: LocalRb::new(4096),
            closed: false,
            control_state: ClientControlState::HttpFirstLine,
            header_buffer: Vec::new(),
            checklist: HttpUpgradeChecklist::default(),
            frame: WebsocketFrameBuilder::default(),
        }
    }

    fn recvin(&mut self) {
        const BUF_SIZE: usize = 4096;
        let mut buffer = [0u8; BUF_SIZE];
        while let Ok(len) = self.socket.read(&mut buffer) {
            if len == 0 {
                self.closed = true;
                break;
            }
            if self.inbox.push_slice(&buffer[0..len]) < len {
                println!("buffer overflow, closing connection");
                self.closed = true;
                break;
            }
            if len < BUF_SIZE {
                break;
            }
        }
    }

    fn poll<MessageType: bitcode::DecodeOwned>(&mut self) -> ClientControllerEvent<MessageType> {
        use ClientControlState::*;
        let old_control_state = self.control_state;

        if let WebSocketFirstTwo = self.control_state {
            let mut buf = [0u8; 2];
            if self.fill_or_incomplete(&mut buf) {
                self.frame.fin = (buf[0] & 0b1000_0000) >> 7 == 1;
                self.frame.rsv = (buf[0] & 0b0111_0000) >> 4;
                self.frame.opcode = buf[0] & 0b0000_1111;
                self.frame.mask = (buf[1] & 0b1000_0000) >> 7 == 1;
                self.frame.len = buf[1] & 0b0111_1111;
                self.control_state = WebSocketExtLength;
            }
        }
        if let WebSocketExtLength = self.control_state {
            if self.frame.len == 126 {
                let mut buf = [0u8; 2];
                if self.fill_or_incomplete(&mut buf) {
                    self.frame.ext_len = Some(u16::from_be_bytes(buf) as usize);
                    self.control_state = WebSocketMasking;
                }
            } else if self.frame.len == 127 {
                let mut buf = [0u8; 8];
                if self.fill_or_incomplete(&mut buf) {
                    self.frame.ext_len = Some(u64::from_be_bytes(buf) as usize);
                    self.control_state = WebSocketMasking;
                }
            } else {
                self.control_state = WebSocketMasking;
            }
        }
        if let WebSocketMasking = self.control_state {
            if self.frame.mask {
                let mut buf = [0u8; 4];
                if self.fill_or_incomplete(&mut buf) {
                    self.frame.mask_key = Some(buf);
                    self.control_state = WebSocketPayload;
                }
            } else {
                self.control_state = WebSocketPayload;
            }
        }
        if let WebSocketPayload = self.control_state {
            if self.inbox.occupied_len() >= self.frame.length() {
                if self.frame.length() > 2048 {
                    self.closed = true;
                    return ClientControllerEvent::Sleep;
                }
                let mut buffer = vec![0u8; self.frame.length()];
                self.inbox.read(&mut buffer).unwrap();
                for i in 0..buffer.len() {
                    // TODO: SIMD optimize this
                    buffer[i] ^= self.frame.mask_key.unwrap()[i % 4];
                }
                self.control_state = WebSocketFirstTwo;
                // TODO: don't unwrap here
                if self.frame.opcode == 2 {
                    // binary, which is the only valid encoding for our purposes
                    let message_decode: MessageType = bitcode::decode(&buffer).unwrap();
                    return ClientControllerEvent::MessageReceived(self.id, message_decode);
                } else if self.frame.opcode == 8 {
                    // close
                    self.send_raw(&[0b1000_1000, 0]); // don't bother with the status code, just send a quick close frame and be done
                    self.closed = true;
                    return ClientControllerEvent::Sleep; // sleep immediately so the connection can be closed
                } else if self.frame.opcode == 9 {
                    self.send_raw(&[0b1000_1010, 0]); // PONG, unmasked 0-byte body
                } // no support for sending pings as they're useless
                return ClientControllerEvent::MaybeUnfinished;
            }
        }

        // HTTP upgrade logic
        // this is about the simplest an HTTP server can be while still maintaining some semblance of usefulness.
        // there's only one valid request line ("GET /game HTTP/1.1\r\n").
        // it rejects bad connections extremely quickly and has very little
        // allocation overhead.
        if let HttpFirstLine = self.control_state {
            if let Some(val) = self.match_or_incomplete(b"GET /game HTTP/1.1\r\n") {
                if val {
                    self.control_state = HttpHeaderName;
                } else {
                    self.error_abort(
                        b"Only GET requests to /game (no trailing slash) are permitted.",
                    );
                }
            }
        }
        if let HttpHeaderNameStartCheckShim = self.control_state {
            if let Some(w) = self.check_whitespace() {
                // we're DONE!
                if w {
                    if self.checklist.will_upgrade() {
                        self.upgrade();
                        self.control_state = WebSocketFirstTwo;
                        return ClientControllerEvent::ClientUpgraded(self.id);
                    } else {
                        self.error_abort(b"All requests must upgrade to websockets.")
                    }
                } else {
                    self.control_state = HttpHeaderName;
                }
            }
        }
        if let HttpHeaderName = self.control_state {
            if self.require_trimmed() {
                if let Some(header_name) = self.match_until_or_incomplete(b':') {
                    self.header_buffer = header_name;
                    self.control_state = HttpHeaderValue;
                }
            }
        }
        if let HttpHeaderValue = self.control_state {
            if self.require_trimmed() {
                if let Some(header_value) = self.match_until_or_incomplete(b'\n') {
                    let header_value = byte_trim(&header_value);
                    if self.header_buffer == b"Connection" && header_value == b"keep-alive, Upgrade"
                    {
                        self.checklist.connection_upgrade = true;
                    } else if self.header_buffer == b"Upgrade" && header_value == b"websocket" {
                        self.checklist.upgrade_websocket = true;
                    } else if self.header_buffer == b"Sec-WebSocket-Key" {
                        let concatenated = format!(
                            "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
                            str::from_utf8(header_value).unwrap()
                        );
                        use base64::prelude::*;
                        use sha1::Digest;
                        let mut hasher = sha1::Sha1::new();
                        hasher.update(concatenated.as_bytes());
                        let result = hasher.finalize();
                        self.checklist.websocket_accept = Some(BASE64_STANDARD.encode(result));
                    }
                    self.control_state = HttpHeaderNameStartCheckShim;
                    return ClientControllerEvent::MaybeUnfinished; // it's possible for old_control_state == HttpHeaderNameStartCheckShim
                                                                   // with unprocessed data; we need to ensure the controller doesn't sleep
                }
            }
        }
        if old_control_state != self.control_state {
            // if there was a state change, we don't want to sleep
            ClientControllerEvent::MaybeUnfinished
        } else {
            ClientControllerEvent::Sleep
        }
    }

    fn error_abort(&mut self, body: &[u8]) {
        println!("error aborting: {}", str::from_utf8(body).unwrap());
        self.send_raw(b"HTTP/1.1 400 Bad Request\r\n\r\n");
        self.send_raw(body);
        self.closed = true;
    }

    fn upgrade(&mut self) {
        self.inbox.clear(); // empty the incoming box of junk bytes
        self.send_raw(b"HTTP/1.1 101 Switching Protocols\r\nConnection: upgrade\r\nUpgrade: websocket\r\nSec-Websocket-Accept: ");
        self.send_raw(self.checklist.websocket_accept.clone().unwrap().as_bytes()); // TODO: don't clone or unwrap here
        self.send_raw(b"\r\n\r\n");
    }

    fn match_or_incomplete(&mut self, data: &[u8]) -> Option<bool> {
        if data.len() <= self.inbox.occupied_len() {
            for i in 0..data.len() {
                if let Some(d) = self.inbox.try_pop() {
                    if d != data[i] {
                        return Some(false);
                    }
                } else {
                    return Some(false);
                }
            }
            Some(true)
        } else {
            None
        }
    }

    fn fill_or_incomplete(&mut self, buf: &mut [u8]) -> bool {
        if self.inbox.occupied_len() >= buf.len() {
            self.inbox.read(buf).unwrap();
            return true;
        }
        false
    }

    fn send_raw(&mut self, data: &[u8]) {
        while let Err(_) = self.outbox.write(data) {
            let _ = self.try_flush();
        }
    }

    fn match_until_or_incomplete(&mut self, until: u8) -> Option<Vec<u8>> {
        let starting_index = self.inbox.read_index();
        let mut count = 0;
        loop {
            let mut buffer = [0u8; 1];
            if self.inbox.read(&mut buffer).ok()? != 1 {
                break;
            }
            if buffer[0] == until {
                let mut result = vec![0u8; count];
                unsafe {
                    self.inbox.set_read_index(starting_index);
                }
                self.inbox
                    .read(&mut result[..])
                    .expect("The previously scanned buffer has magically reallocated itself!");
                self.inbox.skip(1);
                return Some(result);
            }
            count += 1;
        }
        unsafe {
            self.inbox.set_read_index(starting_index);
        }
        None
    }

    fn try_flush(&mut self) -> io::Result<bool> {
        // try to send all the data in outbox through the socket. returns false if the outbox could not be fully
        // emptied, otherwise true.
        const BUF_SIZE: usize = 1024;
        let mut buffer = [0u8; BUF_SIZE];
        loop {
            let amnt = self.outbox.peek_slice(&mut buffer);
            let write_count = self.socket.write(&buffer[0..amnt])?;
            self.outbox.skip(write_count);
            if write_count < BUF_SIZE {
                return Ok(false);
            }
            if amnt < BUF_SIZE {
                return Ok(true);
            }
        }
    }

    fn require_trimmed(&mut self) -> bool {
        while let Some(b) = self.inbox.try_peek() {
            let b = *b;
            if b.is_ascii_whitespace() {
                self.inbox.skip(1);
            } else {
                return true;
            }
        }
        false
    }

    fn check_whitespace(&mut self) -> Option<bool> {
        if let Some(b) = self.inbox.try_peek() {
            return Some(b.is_ascii_whitespace());
        }
        None
    }
}

pub struct Server {
    // misleadingly, the select_bucket is actually passed into poll(), not select(). oops.
    select_bucket: Vec<libc::pollfd>, // cached so we don't have to allocate memory often; this is regenerated when clients are added or dropped
    // the first socket in the select_bucket is always the server socket.
    bucket_clean: bool, // if false, we need to regenerate the bucket before running select
    clients: HashMap<ClientId, Client>,
    top_id: ClientId,
    server_socket: TcpListener,
}

#[derive(Debug)]
pub enum Event<Message> {
    ClientConnected(ClientId),
    ClientDisconnected(ClientId),
    MessageReceived(ClientId, Message),
}

impl Server {
    pub fn new(address: impl net::ToSocketAddrs) -> io::Result<Self> {
        let s = TcpListener::bind(address)?;
        s.set_nonblocking(true)?;
        Ok(Self {
            select_bucket: vec![],
            bucket_clean: false, // always regenerate on the first go
            clients: HashMap::new(),
            top_id: ClientId(0),
            server_socket: s,
        })
    }

    fn generate_bucket(&mut self) {
        self.select_bucket.clear();
        self.select_bucket.push(libc::pollfd {
            fd: self.server_socket.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        });
        for (_, client) in self.clients.iter() {
            self.select_bucket.push(libc::pollfd {
                fd: client.socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }
        self.bucket_clean = true;
    }

    pub fn do_poll<IncomingMessageType: bitcode::DecodeOwned, Context>(
        &mut self,
        context: &mut Context,
        mut message_callback: impl FnMut(ClientId, IncomingMessageType, &mut Server, &mut Context) -> (),
        mut connect_callback: impl FnMut(ClientId, &mut Server, &mut Context) -> (),
        mut disconnect_callback: impl FnMut(ClientId, &mut Context) -> (),
    ) {
        if !self.bucket_clean {
            self.generate_bucket();
        }
        unsafe {
            libc::poll(
                self.select_bucket.as_mut_ptr(),
                self.select_bucket.len() as u64,
                3,
            );
        }
        let mut dispatch_queue = vec![]; // TODO: cache this to avoid allocating
        let mut dispatch_queue_connections = vec![]; // TODO: cache this to avoid allocating
        for (index, (_, client)) in self.clients.iter_mut().enumerate() {
            if 0 != (self.select_bucket[index + 1].revents & libc::POLLIN) {
                // received data!
                client.recvin();
                loop {
                    match client.poll::<IncomingMessageType>() {
                        // poll until the client controller sleeps
                        ClientControllerEvent::Sleep => {
                            break;
                        }
                        ClientControllerEvent::MessageReceived(id, message) => {
                            dispatch_queue.push((id, message));
                        }
                        ClientControllerEvent::ClientUpgraded(id) => {
                            dispatch_queue_connections.push(id);
                        }
                        _ => {}
                    }
                }
            }
        }
        for (id, message) in dispatch_queue {
            message_callback(id, message, self, context);
        }
        for id in dispatch_queue_connections {
            connect_callback(id, self, context);
        }
        for (_, cl) in self.clients.iter_mut() {
            cl.try_flush().unwrap();
        }
        self.clients.retain(|_, client| {
            // prune closed clients
            if client.closed {
                disconnect_callback(client.id, context);
                self.bucket_clean = false;
            }
            !client.closed
        });
        if 0 != (self.select_bucket[0].revents & libc::POLLIN) {
            loop {
                if let Ok((client, _)) = self.server_socket.accept() {
                    client.set_nonblocking(true).unwrap();
                    client.set_nodelay(true).unwrap();
                    self.top_id.0 += 1;
                    self.clients
                        .insert(self.top_id, Client::new_from(self.top_id, client));
                    self.bucket_clean = false;
                } else {
                    break;
                }
            }
        }
    }

    fn make_header(length: usize) -> Vec<u8> {
        let mut header = Vec::with_capacity(14);
        header.push(0b1000_0010);
        if length > 125 {
            if length < 65536 {
                header.extend_from_slice(&(length as u16).to_be_bytes());
            } else {
                header.extend_from_slice(&length.to_be_bytes());
            }
        } else {
            header.push(length as u8); // mask is 0 automatically
        }
        header
    }

    pub fn send_to<MessageType: bitcode::Encode>(
        &mut self,
        client: ClientId,
        message: MessageType,
    ) {
        let client = self.clients.get_mut(&client).unwrap();
        let enc = &bitcode::encode(&message);
        client.send_raw(&Self::make_header(enc.len()));
        client.send_raw(enc);
    }

    pub fn broadcast<MessageType: bitcode::Encode>(&mut self, message: MessageType) {
        let enc = bitcode::encode(&message);
        let header = Self::make_header(enc.len());
        for (_, client) in self.clients.iter_mut() {
            client.send_raw(&header);
            client.send_raw(&enc);
        }
    }

    pub fn close(&mut self, id: ClientId) {
        let cl = self.clients.get_mut(&id).unwrap();
        cl.closed = true;
        cl.send_raw(&[136, 0]);
    }
}

impl Into<common::PlayerId> for ClientId {
    fn into(self) -> common::PlayerId {
        common::PlayerId(self.0 as u64)
    }
}

impl Into<ClientId> for common::PlayerId {
    fn into(self) -> ClientId {
        ClientId(self.0 as usize)
    }
}
