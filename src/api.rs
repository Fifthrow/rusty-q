use kapi;
use kbindings;
use kbindings::*;
use std::convert;
use std::error::Error;
use std::ffi;
use std::fmt;
use std::io;
use std::sync::mpsc;

use types::K;

#[derive(Debug)]
pub struct KError {
    desc: String,
    pub kind: KErr,
}

pub type KResult<T> = Result<T, KError>;

impl Error for KError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl fmt::Display for KError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Kind: {:?}, Desc: {}", self.kind, self.description())
    }
}

impl KError {
    pub fn new(s: String, kind: KErr) -> Self {
        KError { desc: s, kind }
    }
}

impl convert::From<io::Error> for KError {
    fn from(err: io::Error) -> KError {
        KError {
            desc: err.description().to_string(),
            kind: KErr::IOErr,
        }
    }
}

impl convert::From<mpsc::RecvError> for KError {
    fn from(err: mpsc::RecvError) -> KError {
        KError {
            desc: err.description().to_string(),
            kind: KErr::RecvErr,
        }
    }
}

#[derive(Debug)]
pub enum KErr {
    ConnectionFailed,
    AuthenticationFailed,
    QueryFailed,
    SocketClosed,
    SocketTimeout,
    IOErr,
    RecvErr,
    SendErr,
    EncodeFailed,
    DecodeFailed,
    CorruptData,
    BadConfig,
    Generic,
    WrongType,
}

#[allow(dead_code)]
pub struct Handle {
    host: String,
    port: i32,
    username: String,
    handle: i32,
}

impl Handle {
    pub fn connect(host: &str, port: i32, username: &str) -> Result<Handle, Box<dyn Error>> {
        let chost = ffi::CString::new(host)?;
        let cuser = ffi::CString::new(username)?;
        let handle = match unsafe { kapi::khpu(chost.as_ptr(), port, cuser.as_ptr()) } {
            h if h < 0 => {
                return Err(Box::new(KError::new(
                    "Could not connect".to_string(),
                    KErr::ConnectionFailed,
                )))
            }
            0 => {
                return Err(Box::new(KError::new(
                    "Wrong credentials".to_string(),
                    KErr::AuthenticationFailed,
                )))
            }
            h => h,
        };
        let cq = ffi::CString::new("")?;
        unsafe { kapi::k(handle, cq.as_ptr(), kvoid()) }; //memory allocation
        Ok(Handle {
            host: host.to_string(),
            username: username.to_string(),
            port,
            handle,
        })
    }

    pub fn query(&self, query: &str) -> Result<KOwned, Box<dyn Error>> {
        let cquery = ffi::CString::new(query)?;
        let kptr = unsafe { kapi::k(self.handle, cquery.as_ptr(), kvoid()) };
        if kptr.is_null() {
            return Err(Box::new(KError::new(
                "Query failed".to_string(),
                KErr::QueryFailed,
            )));
        }
        Ok(unsafe { KOwned(&*kptr) })
    }

    pub unsafe fn async_pub(&self, callback: &str, topic: *const K, object: *const K) {
        let cbk = ffi::CString::new(callback).unwrap();
        let kptr = kapi::k(-self.handle, cbk.as_ptr(), topic, object, kvoid());
        if (&*kptr).t == -128 {
            println!("Error sending async data"); // Err(Box::new(KError::new("Query failed".to_string(), KErr::QueryFailed)));
            KOwned(&*kptr);
        }
    }

    pub unsafe fn sync_get(
        &self,
        function: &str,
        object: *const K,
    ) -> Result<KOwned, Box<dyn Error>> {
        let cbk = ffi::CString::new(function)?;
        let kptr = kapi::k(self.handle, cbk.as_ptr(), object, kvoid());
        if kptr.is_null() {
            return Err(Box::new(KError::new(
                "Query failed".to_string(),
                KErr::QueryFailed,
            )));
        }
        Ok(KOwned(&*kptr))
    }

    pub fn close(&self) {
        unsafe { kapi::kclose(self.handle) };
    }
}

pub fn serialize(k: &KOwned) -> KResult<KOwned> {
    let ser = unsafe { &*kapi::b9(3, k.0) };
    if kbindings::valid_stream(ser) {
        Ok(KOwned(ser))
    } else {
        Err(KError::new(
            "Invalid serialization".to_string(),
            KErr::EncodeFailed,
        ))
    }
}

pub fn deserialize(ser: &KOwned) -> KResult<KOwned> {
    if kbindings::valid_stream(ser.0) {
        Ok(KOwned(kbindings::deserial(ser.0)))
    } else {
        Err(KError::new(
            "Invalid deserialization".to_string(),
            KErr::DecodeFailed,
        ))
    }
}
