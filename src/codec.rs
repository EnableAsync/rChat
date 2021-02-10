#![allow(dead_code)]

use std::io;

use crate::chat_utils::to_io_error;
use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

/// Client request
#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum ChatRequest {
    /// Set NickName
    NickName(String),
    /// List rooms
    List,
    /// Join rooms
    Join(String),
    /// Send message
    Message(String),
    /// Ping
    Ping,
}

/// Server response
#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum ChatResponse {
    Ping,

    /// List of rooms
    Rooms(Vec<String>),

    /// Joined
    Joined(String),

    /// Message
    Message(String),

    /// Set nickname successfully
    SetNickName(String),
}

/// Codec for Client -> Server transport
pub struct ChatCodec;

impl Decoder for ChatCodec {
    type Item = ChatRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(
                serde_cbor::from_slice::<ChatRequest>(&buf).map_err(to_io_error)?,
            ))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<ChatResponse> for ChatCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: ChatResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg_vec = serde_cbor::to_vec(&msg).unwrap();
        let msg_ref: &[u8] = msg_vec.as_ref();
        match msg {
            ChatResponse::Ping => {}
            _ => {
                println!("msg: {:?}", msg);
                println!("cbor: {:?}", msg_ref);
            }
        }

        // let obj:ChatResponse = serde_cbor::from_slice(&buf).unwrap();
        // println!("from cbor: {:?}", obj);

        // let msg = json::to_string(&msg).unwrap();
        // let msg_ref: &[u8] = msg.as_ref();
        // println!("json: {:?}", msg);

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}

/// Codec for Server -> Client transport
pub struct ClientChatCodec;

impl Decoder for ClientChatCodec {
    type Item = ChatResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(
                serde_cbor::from_slice::<ChatResponse>(&buf).map_err(to_io_error)?,
            ))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<ChatRequest> for ClientChatCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: ChatRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = serde_cbor::to_vec(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
