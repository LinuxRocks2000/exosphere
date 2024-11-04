use wasm_bindgen::prelude::*;
use common::comms::*;
use common::protocol::Protocol;
use common::VERSION;


#[wasm_bindgen]
pub enum KeyEvent {
    KeyUp,
    KeyDown
}

#[wasm_bindgen]
pub enum MouseEvent {
    Move,
    Up,
    Down
}


#[wasm_bindgen(module="/core.js")]
extern "C" {
    fn alert(s : &str);
    fn register_listeners(
        tick : &Closure<dyn FnMut()>,
        key : &Closure<dyn FnMut(String, KeyEvent)>,
        mouse : &Closure<dyn FnMut(f32, f32, MouseEvent)>,
        websocket : &Closure<dyn FnMut(Vec<u8>)>);
    fn send_ws(data : Vec<u8>);
}


fn send(message : ClientMessage) {
    let mut buf = vec![0u8; message.size()];
    message.encode_into(&mut buf);
    send_ws(buf);
}


#[wasm_bindgen]
pub fn entrypoint() {
    let mut has_tested = false;
    let tick = Box::new(Closure::new(|| {
        
    }));
    let key = Box::new(Closure::new(|key, evt| {
        alert("key!");
    }));
    let mouse = Box::new(Closure::new(|mx, my, evt| {
        alert("mouse");
    }));
    let websocket = Box::new(Closure::new(move |d : Vec<u8>| {
        let message = ServerMessage::decode_from(&d);
        if let Ok(ref msg) = message {
            if has_tested {

            }
            else {
                if let ServerMessage::Test(exostring, 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION) = msg {
                    if exostring == "EXOSPHERE" {
                        send(ClientMessage::Test("EXOSPHERE".to_string(), 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION));
                        has_tested = true;
                        return;
                    }
                }
                alert(&format!("server failed verification: {:?}", message));
            }
        }
        else {
            alert("error");
        }
    }));
    register_listeners( // register_listeners is glued into javascript, presumably around memory protections. dropping the closures doesn't do anything on the WASM side but
        // the javascript side breaks (because it's trying to call closures that don't exist)
        // if you know a better way to intentionally leak memory over the wasm boundary, PLEASE pr it in.
        Box::leak(tick),
        Box::leak(key),
        Box::leak(mouse),
        Box::leak(websocket)
    );
}