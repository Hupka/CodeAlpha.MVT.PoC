use serde::{Deserialize, Serialize};

use super::types::{Event, Request, Response};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Event(Event),
    Request(Request),
    Response(Response),
    None,
}
