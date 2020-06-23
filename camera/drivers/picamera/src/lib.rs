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
use camera_core as core;

#[pyclass]
pub struct PyStream {
}

impl PyStream {
    fn new(width: usize, height: usize, dest: &str) -> PyResult<Self> {
        Ok(PyStream{})
    }
    
    fn send_frame(&mut self, data: &[u8]) {
    }
}

#[pymodule]
fn py_stream(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStream>()
}

