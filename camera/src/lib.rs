#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate pyo3;
use pyo3::prelude::*;
use pyo3::exceptions::RuntimeError;
use pyo3::{PyErr, types::{PyString, PyBytes}};
use crossbeam::channel::{Sender, Receiver};
use std::{ptr, ffi::{c_void, CStr}};
use std::path::PathBuf;
use std::os::raw::c_char;
use anyhow::{Result, Error};
use futures::StreamExt;
use std::{net::SocketAddr, sync::{Arc, Weak, Mutex}};
use quinn::{ClientConfig, ClientConfigBuilder};

#[pyclass]
pub struct PyStream {
    stop: Sender<()>,
    inner: Arc<StreamInner>,
}

impl PyStream {
    fn new(width: usize, height: usize, dest: &str) -> Result<Self> {
    }
    
    fn send_frame(&mut self, data: &[u8]) {
    }
}

#[pymodule]
fn py_stream(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStream>()
}

// TODO: refactor Stream into mod
// TODO: write test code
