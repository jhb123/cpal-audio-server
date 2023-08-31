#[macro_use] extern crate rocket;

use rocket::{http::RawStr, request::FromRequest};
use rocket_ws as ws;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// #[get("/echo")]
// fn echo() -> &'static str {
//     "Hello, world!"
// }


// #[get("/<dynamic_test>")]
// fn dynamic_echo(dynamic_test: String) -> String {
//     dynamic_test
// }

// #[get("/echo?<word>")]
// fn echo(word: &RawStr) -> String {
//     word.to_string()
// }

#[get("/echo?<word>&<number>")]
fn echo_repeat(word: String, number: Option<usize> ) -> String {
    match number {
        Some(n) => {word.to_string().repeat(n)},
        None => {word.to_string()},
    }
    
}

#[get("/echo?stream")]
fn echo_stream(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, echo_repeat,echo_stream])
}
