use std::fmt;
use std::convert::{Into, From};
use sha1;


use self::OpCode::*;
/// Operation codes as part of rfc6455.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum OpCode {
    /// Indicates a continuation frame of a fragmented message.
    Continue,
    /// Indicates a text data frame.
    Text,
    /// Indicates a binary data frame.
    Binary,
    /// Indicates a close control frame.
    Close,
    /// Indicates a ping control frame.
    Ping,
    /// Indicates a pong control frame.
    Pong,
    /// Indicates an invalid opcode was received.
    Bad,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Continue   =>   write!(f, "CONTINUE"),
            Text       =>   write!(f, "TEXT"),
            Binary     =>   write!(f, "BINARY"),
            Close      =>   write!(f, "CLOSE"),
            Ping       =>   write!(f, "PING"),
            Pong       =>   write!(f, "PONG"),
            Bad        =>   write!(f, "BAD"),
        }
    }
}

impl Into<u8> for OpCode {

    fn into(self) -> u8 {
        match self {
            Continue   =>   0,
            Text       =>   1,
            Binary     =>   2,
            Close      =>   8,
            Ping       =>   9,
            Pong       =>   10,
            Bad        => {
                debug_assert!(false, "Attempted to convert invalid opcode to u8. This is a bug.");
                8  // if this somehow happens, a close frame will help us tear down quickly
            }
        }
    }
}

impl From<u8> for OpCode {

    fn from(byte: u8) -> OpCode {
        match byte {
            0   =>   Continue,
            1   =>   Text,
            2   =>   Binary,
            8   =>   Close,
            9   =>   Ping,
            10  =>   Pong,
            _   =>   Bad
        }
    }
}

use self::CloseCode::*;
/// Status code used to indicate why an endpoint is closing the `WebSocket` connection.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CloseCode {
    /// Indicates a normal closure, meaning that the purpose for
    /// which the connection was established has been fulfilled.
    Normal,
    /// Indicates that an endpoint is "going away", such as a server
    /// going down or a browser having navigated away from a page.
    Away,
    /// Indicates that an endpoint is terminating the connection due
    /// to a protocol error.
    Protocol,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a type of data it cannot accept (e.g., an
    /// endpoint that understands only text data MAY send this if it
    /// receives a binary message).
    Unsupported,
    /// Indicates that no status code was included in a closing frame. This
    /// close code makes it possible to use a single method, `on_close` to
    /// handle even cases where no close code was provided.
    Status,
    /// Indicates an abnormal closure. If the abnormal closure was due to an
    /// error, this close code will not be used. Instead, the `on_error` method
    /// of the handler will be called with the error. However, if the connection
    /// is simply dropped, without an error, this close code will be sent to the
    /// handler.
    Abnormal,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received data within a message that was not
    /// consistent with the type of the message (e.g., non-UTF-8 [RFC3629]
    /// data within a text message).
    Invalid,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that violates its policy.  This
    /// is a generic status code that can be returned when there is no
    /// other more suitable status code (e.g., Unsupported or Size) or if there
    /// is a need to hide specific details about the policy.
    Policy,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that is too big for it to
    /// process.
    Size,
    /// Indicates that an endpoint (client) is terminating the
    /// connection because it has expected the server to negotiate one or
    /// more extension, but the server didn't return them in the response
    /// message of the WebSocket handshake.  The list of extensions that
    /// are needed should be given as the reason for closing.
    /// Note that this status code is not used by the server, because it
    /// can fail the WebSocket handshake instead.
    Extension,
    /// Indicates that a server is terminating the connection because
    /// it encountered an unexpected condition that prevented it from
    /// fulfilling the request.
    Error,
    /// Indicates that the server is restarting. A client may choose to reconnect,
    /// and if it does, it should use a randomized delay of 5-30 seconds between attempts.
    Restart,
    /// Indicates that the server is overloaded and the client should either connect
    /// to a different IP (when multiple targets exist), or reconnect to the same IP
    /// when a user has performed an action.
    Again,
    #[doc(hidden)]
    Tls,
    #[doc(hidden)]
    Empty,
    #[doc(hidden)]
    Other(u16),
}

impl Into<u16> for CloseCode {

    fn into(self) -> u16 {
        match self {
           Normal        =>   1000,
           Away          =>   1001,
           Protocol      =>   1002,
           Unsupported   =>   1003,
           Status        =>   1005,
           Abnormal      =>   1006,
           Invalid       =>   1007,
           Policy        =>   1008,
           Size          =>   1009,
           Extension     =>   1010,
           Error         =>   1011,
           Restart       =>   1012,
           Again         =>   1013,
           Tls           =>   1015,
           Empty         =>   0,
           Other(code)   =>   code,
        }
    }
}

impl From<u16> for CloseCode {

    fn from(code: u16) -> CloseCode {
        match code {
            1000 => Normal,
            1001 => Away,
            1002 => Protocol,
            1003 => Unsupported,
            1005 => Status,
            1006 => Abnormal,
            1007 => Invalid,
            1008 => Policy,
            1009 => Size,
            1010 => Extension,
            1011 => Error,
            1012 => Restart,
            1013 => Again,
            1015 => Tls,
            0    => Empty,
            _ => Other(code),
        }
    }
}


static WS_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
static BASE64: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";


// TODO: hash is always same size, we dont need String
pub(crate) fn hash_key(key: &[u8]) -> String {
    let mut hasher = sha1::Sha1::new();

    hasher.update(key);
    hasher.update(WS_GUID.as_bytes());

    encode_base64(&hasher.digest().bytes())
}


// This code is based on rustc_serialize base64 STANDARD
fn encode_base64(data: &[u8]) -> String {
    let len = data.len();
    let mod_len = len % 3;

    let mut encoded = vec![b'='; (len + 2) / 3 * 4];
    {
        let mut in_iter = data[..len - mod_len].iter().map(|&c| u32::from(c));
        let mut out_iter = encoded.iter_mut();

        let enc = |val| BASE64[val as usize];
        let mut write = |val| *out_iter.next().unwrap() = val;

        while let (Some(one), Some(two), Some(three)) = (in_iter.next(), in_iter.next(), in_iter.next()) {
            let g24 = one << 16 | two << 8 | three;
            write(enc((g24 >> 18) & 63));
            write(enc((g24 >> 12) & 63));
            write(enc((g24 >> 6 ) & 63));
            write(enc(g24 & 63));
        }

        match mod_len {
            1 => {
                let pad = u32::from(data[len-1]) << 16;
                write(enc((pad >> 18) & 63));
                write(enc((pad >> 12) & 63));
            }
            2 => {
                let pad = u32::from(data[len-2]) << 16 | u32::from(data[len-1]) << 8;
                write(enc((pad >> 18) & 63));
                write(enc((pad >> 12) & 63));
                write(enc((pad >> 6) & 63));
            }
            _ => (),
        }
    }

    String::from_utf8(encoded).unwrap()
}


mod test {
    #![allow(unused_imports, unused_variables, dead_code)]
    use super::*;

    #[test]
    fn opcode_from_u8() {
        let byte = 2u8;
        assert_eq!(OpCode::from(byte), OpCode::Binary);
    }

    #[test]
    fn opcode_into_u8() {
        let text = OpCode::Text;
        let byte: u8 = text.into();
        assert_eq!(byte, 1u8);
    }

    #[test]
    fn closecode_from_u16() {
        let byte = 1008u16;
        assert_eq!(CloseCode::from(byte), CloseCode::Policy);
    }

    #[test]
    fn closecode_into_u16() {
        let text = CloseCode::Away;
        let byte: u16 = text.into();
        assert_eq!(byte, 1001u16);
    }
}