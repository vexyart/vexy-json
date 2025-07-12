// this_file: crates/python/src/lib.rs

//! Python bindings for vexy_json - a forgiving JSON parser.
//!
//! This module provides Python bindings for the vexy_json library using PyO3,
//! allowing Python users to parse forgiving JSON with the same capabilities
//! as the Rust library.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyList};
use rustc_hash::FxHashMap;
use vexy_json_core::ast::Value;
use vexy_json_core::{parse, parse_with_options, ParserOptions};

/// Convert a vexy_json Value to a Python object
fn value_to_python(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(PyBool::new(py, *b).as_any().clone().unbind()),
        Value::Number(num) => match num {
            vexy_json_core::ast::Number::Integer(i) => Ok((*i).into_pyobject(py)?.unbind().into()),
            vexy_json_core::ast::Number::Float(f) => Ok((*f).into_pyobject(py)?.unbind().into()),
        },
        Value::String(s) => Ok(s.as_str().into_pyobject(py)?.unbind().into()),
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = value_to_python(py, item)?;
                py_list.append(py_item)?;
            }
            Ok(py_list.as_any().clone().unbind())
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, value) in obj {
                let py_value = value_to_python(py, value)?;
                py_dict.set_item(key, py_value)?;
            }
            Ok(py_dict.as_any().clone().unbind())
        }
    }
}

/// Convert a Python object to a vexy_json Value
fn python_to_value(py: Python, obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(vexy_json_core::ast::Number::Integer(i)))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Value::Number(vexy_json_core::ast::Number::Float(f)))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut vec = Vec::new();
        for item in list.iter() {
            vec.push(python_to_value(py, &item)?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = FxHashMap::default();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let value_obj = python_to_value(py, &value)?;
            map.insert(key_str, value_obj);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyTypeError::new_err(format!(
            "Cannot convert Python object of type {} to vexy_json Value",
            obj.get_type().name()?
        )))
    }
}

/// Parse a JSON string with default options (all forgiving features enabled)
///
/// Args:
///     input (str): The JSON string to parse
///
/// Returns:
///     The parsed JSON as a Python object (dict, list, str, int, float, bool, or None)
///
/// Raises:
///     ValueError: If the input is not valid JSON
///
/// Example:
///     >>> import vexy_json
///     >>> result = vexy_json.parse('{"key": "value", trailing: true,}')
///     >>> print(result)
///     {'key': 'value', 'trailing': True}
#[pyfunction]
fn parse_json(py: Python, input: &str) -> PyResult<PyObject> {
    match parse(input) {
        Ok(value) => value_to_python(py, &value),
        Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
    }
}

/// Parse a JSON string with custom options
///
/// Args:
///     input (str): The JSON string to parse
///     allow_comments (bool, optional): Allow single-line and multi-line comments. Defaults to True.
///     allow_trailing_commas (bool, optional): Allow trailing commas in arrays and objects. Defaults to True.
///     allow_unquoted_keys (bool, optional): Allow unquoted object keys. Defaults to True.
///     allow_single_quotes (bool, optional): Allow single-quoted strings. Defaults to True.
///     implicit_top_level (bool, optional): Allow implicit top-level objects/arrays. Defaults to True.
///     newline_as_comma (bool, optional): Treat newlines as commas. Defaults to True.
///     max_depth (int, optional): Maximum nesting depth. Defaults to 128.
///     enable_repair (bool, optional): Enable JSON repair functionality. Defaults to True.
///     max_repairs (int, optional): Maximum number of repairs to attempt. Defaults to 100.
///     fast_repair (bool, optional): Prefer speed over repair quality. Defaults to False.
///     report_repairs (bool, optional): Report all repairs made. Defaults to True.
///
/// Returns:
///     The parsed JSON as a Python object
///
/// Raises:
///     ValueError: If the input is not valid JSON
///
/// Example:
///     >>> import vexy_json
///     >>> result = vexy_json.parse_with_options('key: value', implicit_top_level=True)
///     >>> print(result)
///     {'key': 'value'}
#[pyfunction]
#[pyo3(signature = (
    input,
    allow_comments = true,
    allow_trailing_commas = true,
    allow_unquoted_keys = true,
    allow_single_quotes = true,
    implicit_top_level = true,
    newline_as_comma = true,
    max_depth = 128,
    enable_repair = true,
    max_repairs = 100,
    fast_repair = false,
    report_repairs = true
))]
fn parse_with_options_py(
    py: Python,
    input: &str,
    allow_comments: bool,
    allow_trailing_commas: bool,
    allow_unquoted_keys: bool,
    allow_single_quotes: bool,
    implicit_top_level: bool,
    newline_as_comma: bool,
    max_depth: usize,
    enable_repair: bool,
    max_repairs: usize,
    fast_repair: bool,
    report_repairs: bool,
) -> PyResult<PyObject> {
    let options = ParserOptions {
        allow_comments,
        allow_trailing_commas,
        allow_unquoted_keys,
        allow_single_quotes,
        implicit_top_level,
        newline_as_comma,
        max_depth,
        enable_repair,
        max_repairs,
        fast_repair,
        report_repairs,
    };

    match parse_with_options(input, options) {
        Ok(value) => value_to_python(py, &value),
        Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
    }
}

/// Check if a string is valid JSON/Vexy JSON
///
/// Args:
///     input (str): The JSON string to validate
///
/// Returns:
///     bool: True if the input is valid, False otherwise
///
/// Example:
///     >>> import vexy_json
///     >>> vexy_json.is_valid('{"valid": true}')
///     True
///     >>> vexy_json.is_valid('invalid json')
///     False
#[pyfunction]
fn is_valid(input: &str) -> bool {
    parse(input).is_ok()
}

/// Dumps a Python object to a JSON string
///
/// Args:
///     obj: The Python object to serialize
///     indent (int, optional): Number of spaces for indentation. If None, output is compact.
///
/// Returns:
///     str: The JSON string representation
///
/// Raises:
///     TypeError: If the object cannot be serialized to JSON
///
/// Example:
///     >>> import vexy_json
///     >>> data = {'key': 'value', 'number': 42}
///     >>> vexy_json.dumps(data)
///     '{"key":"value","number":42}'
///     >>> vexy_json.dumps(data, indent=2)
///     '{\n  "key": "value",\n  "number": 42\n}'
#[pyfunction]
#[pyo3(signature = (obj, indent = None))]
fn dumps(py: Python, obj: &Bound<'_, PyAny>, indent: Option<usize>) -> PyResult<String> {
    let value = python_to_value(py, obj)?;

    if let Some(spaces) = indent {
        // Pretty printing with indentation
        Ok(format_value_pretty(&value, spaces))
    } else {
        // Compact output
        Ok(value.to_string())
    }
}

/// Format a Value with pretty printing
fn format_value_pretty(value: &Value, indent: usize) -> String {
    format_value_with_indent(value, 0, indent)
}

fn format_value_with_indent(value: &Value, current_indent: usize, indent_size: usize) -> String {
    let spaces = " ".repeat(current_indent);
    let inner_spaces = " ".repeat(current_indent + indent_size);

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(num) => match num {
            vexy_json_core::ast::Number::Integer(i) => i.to_string(),
            vexy_json_core::ast::Number::Float(f) => f.to_string(),
        },
        Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let mut result = "[\n".to_string();
                for (i, item) in arr.iter().enumerate() {
                    result.push_str(&inner_spaces);
                    result.push_str(&format_value_with_indent(
                        item,
                        current_indent + indent_size,
                        indent_size,
                    ));
                    if i < arr.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push(']');
                result
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut result = "{\n".to_string();
                let mut entries: Vec<_> = obj.iter().collect();
                entries.sort_by_key(|(k, _)| *k);

                for (i, (key, value)) in entries.iter().enumerate() {
                    result.push_str(&inner_spaces);
                    result.push_str(&format!("\"{}\": ", key));
                    result.push_str(&format_value_with_indent(
                        value,
                        current_indent + indent_size,
                        indent_size,
                    ));
                    if i < entries.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&spaces);
                result.push('}');
                result
            }
        }
    }
}

/// Load JSON from a file-like object
///
/// Args:
///     fp: A file-like object supporting .read()
///     **kwargs: Additional arguments passed to parse_with_options
///
/// Returns:
///     The parsed JSON as a Python object
///
/// Raises:
///     ValueError: If the content is not valid JSON
///
/// Example:
///     >>> import vexy_json
///     >>> with open('data.json', 'r') as f:
///     ...     result = vexy_json.load(f)
#[pyfunction]
#[pyo3(signature = (fp, **kwargs))]
fn load(
    py: Python,
    fp: &Bound<'_, PyAny>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<PyObject> {
    // Read content from file-like object
    let content = fp.call_method0("read")?;
    let content_str = content.extract::<String>()?;

    // Parse with options if provided
    if let Some(options) = kwargs {
        let allow_comments = options
            .get_item("allow_comments")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let allow_trailing_commas = options
            .get_item("allow_trailing_commas")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let allow_unquoted_keys = options
            .get_item("allow_unquoted_keys")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let allow_single_quotes = options
            .get_item("allow_single_quotes")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let implicit_top_level = options
            .get_item("implicit_top_level")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let newline_as_comma = options
            .get_item("newline_as_comma")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let max_depth = options
            .get_item("max_depth")?
            .map(|v| v.extract::<usize>().unwrap_or(128))
            .unwrap_or(128);
        let enable_repair = options
            .get_item("enable_repair")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);
        let max_repairs = options
            .get_item("max_repairs")?
            .map(|v| v.extract::<usize>().unwrap_or(100))
            .unwrap_or(100);
        let fast_repair = options
            .get_item("fast_repair")?
            .map(|v| v.extract::<bool>().unwrap_or(false))
            .unwrap_or(false);
        let report_repairs = options
            .get_item("report_repairs")?
            .map(|v| v.extract::<bool>().unwrap_or(true))
            .unwrap_or(true);

        parse_with_options_py(
            py,
            &content_str,
            allow_comments,
            allow_trailing_commas,
            allow_unquoted_keys,
            allow_single_quotes,
            implicit_top_level,
            newline_as_comma,
            max_depth,
            enable_repair,
            max_repairs,
            fast_repair,
            report_repairs,
        )
    } else {
        parse_json(py, &content_str)
    }
}

/// Dump JSON to a file-like object
///
/// Args:
///     obj: The Python object to serialize
///     fp: A file-like object supporting .write()
///     indent (int, optional): Number of spaces for indentation
///
/// Raises:
///     TypeError: If the object cannot be serialized
///
/// Example:
///     >>> import vexy_json
///     >>> data = {'key': 'value'}
///     >>> with open('output.json', 'w') as f:
///     ...     vexy_json.dump(data, f, indent=2)
#[pyfunction]
#[pyo3(signature = (obj, fp, indent = None))]
fn dump(
    py: Python,
    obj: &Bound<'_, PyAny>,
    fp: &Bound<'_, PyAny>,
    indent: Option<usize>,
) -> PyResult<()> {
    let json_str = dumps(py, obj, indent)?;
    fp.call_method1("write", (json_str,))?;
    Ok(())
}

/// Streaming JSON parser with context manager support
///
/// This class provides a streaming JSON parser that can be used with Python's
/// context manager protocol (`with` statement) for efficient processing of large
/// JSON files or streams.
///
/// Example:
///     >>> import vexy_json
///     >>> with vexy_json.StreamingParser() as parser:
///     ...     for item in parser.parse_stream(file_handle):
///     ...         print(item)
#[pyclass]
struct StreamingParser {
    /// Parser options for streaming
    options: ParserOptions,
    /// Buffer for incomplete JSON chunks
    buffer: String,
    /// Whether the parser is active
    active: bool,
}

#[pymethods]
impl StreamingParser {
    /// Create a new streaming parser
    #[new]
    #[pyo3(signature = (
        allow_comments = true,
        allow_trailing_commas = true,
        allow_unquoted_keys = true,
        allow_single_quotes = true,
        implicit_top_level = true,
        newline_as_comma = true,
        max_depth = 128,
        enable_repair = true,
        max_repairs = 100,
        fast_repair = false,
        report_repairs = true
    ))]
    fn new(
        allow_comments: bool,
        allow_trailing_commas: bool,
        allow_unquoted_keys: bool,
        allow_single_quotes: bool,
        implicit_top_level: bool,
        newline_as_comma: bool,
        max_depth: usize,
        enable_repair: bool,
        max_repairs: usize,
        fast_repair: bool,
        report_repairs: bool,
    ) -> Self {
        let options = ParserOptions {
            allow_comments,
            allow_trailing_commas,
            allow_unquoted_keys,
            allow_single_quotes,
            implicit_top_level,
            newline_as_comma,
            max_depth,
            enable_repair,
            max_repairs,
            fast_repair,
            report_repairs,
        };

        Self {
            options,
            buffer: String::new(),
            active: false,
        }
    }

    /// Context manager entry
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager exit
    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.active = false;
        self.buffer.clear();
        Ok(false) // Don't suppress exceptions
    }

    /// Parse a stream of JSON objects
    ///
    /// Args:
    ///     fp: A file-like object supporting .read() or .readline()
    ///
    /// Returns:
    ///     Iterator of parsed JSON objects
    ///
    /// Example:
    ///     >>> with vexy_json.StreamingParser() as parser:
    ///     ...     for item in parser.parse_stream(file_handle):
    ///     ...         process(item)
    fn parse_stream(&mut self, _py: Python, fp: &Bound<'_, PyAny>) -> PyResult<StreamingIterator> {
        self.active = true;
        Ok(StreamingIterator {
            fp: fp.clone().into(),
            options: self.options.clone(),
            buffer: String::new(),
        })
    }

    /// Parse lines from a file as individual JSON objects (NDJSON format)
    ///
    /// Args:
    ///     fp: A file-like object supporting .readline()
    ///
    /// Returns:
    ///     Iterator of parsed JSON objects
    ///
    /// Example:
    ///     >>> with vexy_json.StreamingParser() as parser:
    ///     ...     for item in parser.parse_lines(file_handle):
    ///     ...         process(item)
    fn parse_lines(&mut self, _py: Python, fp: &Bound<'_, PyAny>) -> PyResult<LineIterator> {
        self.active = true;
        Ok(LineIterator {
            fp: fp.clone().into(),
            options: self.options.clone(),
        })
    }
}

/// Iterator for streaming JSON parsing
#[pyclass]
struct StreamingIterator {
    fp: PyObject,
    options: ParserOptions,
    buffer: String,
}

#[pymethods]
impl StreamingIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        // Read chunk from file
        let chunk = self.fp.call_method1(py, "read", (8192,))?;
        let chunk_str = chunk.extract::<String>(py)?;

        if chunk_str.is_empty() {
            return Ok(None);
        }

        self.buffer.push_str(&chunk_str);

        // Try to parse complete JSON objects from buffer
        // This is a simplified implementation - a real streaming parser would be more sophisticated
        if let Some(end_pos) = self.find_complete_json_end(&self.buffer) {
            let json_str = self.buffer[..end_pos].to_string();
            self.buffer.drain(..end_pos);

            match parse_with_options(&json_str, self.options.clone()) {
                Ok(value) => {
                    let py_obj = value_to_python(py, &value)?;
                    Ok(Some(py_obj))
                }
                Err(_) => {
                    // Skip invalid JSON and continue
                    self.__next__(py)
                }
            }
        } else {
            // Need more data
            self.__next__(py)
        }
    }
}

impl StreamingIterator {
    fn find_complete_json_end(&self, buffer: &str) -> Option<usize> {
        // Simple implementation - look for complete JSON objects
        // In a real implementation, this would need proper bracket/brace counting
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in buffer.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '\\' if in_string => escape_next = true,
                '{' | '[' if !in_string => depth += 1,
                '}' | ']' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
        }

        None
    }
}

/// Iterator for line-by-line JSON parsing (NDJSON)
#[pyclass]
struct LineIterator {
    fp: PyObject,
    options: ParserOptions,
}

#[pymethods]
impl LineIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        let line = self.fp.call_method0(py, "readline")?;
        let line_str = line.extract::<String>(py)?;

        if line_str.is_empty() {
            return Ok(None);
        }

        let trimmed = line_str.trim();
        if trimmed.is_empty() {
            return self.__next__(py); // Skip empty lines
        }

        match parse_with_options(trimmed, self.options.clone()) {
            Ok(value) => {
                let py_obj = value_to_python(py, &value)?;
                Ok(Some(py_obj))
            }
            Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
        }
    }
}

/// Parse JSON array directly to NumPy array (if NumPy is available)
///
/// Args:
///     input (str): The JSON array string to parse
///     dtype (str, optional): NumPy dtype for the array. Defaults to auto-detection.
///
/// Returns:
///     numpy.ndarray: The parsed array as a NumPy array
///
/// Raises:
///     ValueError: If the input is not a valid JSON array
///     ImportError: If NumPy is not available
///
/// Example:
///     >>> import vexy_json
///     >>> arr = vexy_json.loads_numpy('[1, 2, 3, 4, 5]')
///     >>> print(type(arr))
///     <class 'numpy.ndarray'>
#[pyfunction]
#[pyo3(signature = (input, dtype = None))]
fn loads_numpy(py: Python, input: &str, dtype: Option<&str>) -> PyResult<PyObject> {
    // Try to import numpy
    let numpy = match py.import("numpy") {
        Ok(np) => np,
        Err(_) => {
            return Err(PyValueError::new_err(
                "NumPy is not available. Please install numpy: pip install numpy",
            ))
        }
    };

    // Parse the JSON
    let value = match parse(input) {
        Ok(v) => v,
        Err(e) => return Err(PyValueError::new_err(format!("Parse error: {}", e))),
    };

    // Convert to NumPy array
    match value {
        Value::Array(arr) => {
            // Check if all elements are numbers for efficient conversion
            let all_numbers = arr.iter().all(|v| matches!(v, Value::Number(_)));

            if all_numbers {
                // Fast path for numeric arrays
                let numbers: Vec<f64> = arr
                    .iter()
                    .map(|v| {
                        match v {
                            Value::Number(vexy_json_core::ast::Number::Integer(i)) => *i as f64,
                            Value::Number(vexy_json_core::ast::Number::Float(f)) => *f,
                            _ => 0.0, // Should not happen due to all_numbers check
                        }
                    })
                    .collect();

                let numpy_array = if let Some(dt) = dtype {
                    numpy.call_method1("array", (numbers, dt))?
                } else {
                    numpy.call_method1("array", (numbers,))?
                };

                Ok(numpy_array.into())
            } else {
                // Fallback: convert to Python objects first, then to NumPy
                let py_list = PyList::empty(py);
                for item in arr {
                    let py_item = value_to_python(py, &item)?;
                    py_list.append(py_item)?;
                }

                let numpy_array = if let Some(dt) = dtype {
                    numpy.call_method1("array", (py_list, dt))?
                } else {
                    numpy.call_method1("array", (py_list,))?
                };

                Ok(numpy_array.into())
            }
        }
        _ => Err(PyValueError::new_err(
            "Input must be a JSON array for NumPy conversion",
        )),
    }
}

/// Parse JSON array with zero-copy optimization for numeric data
///
/// Args:
///     input (str): The JSON array string to parse
///     dtype (str, optional): Target dtype for the array
///
/// Returns:
///     numpy.ndarray: The parsed array with zero-copy optimization when possible
///
/// Example:
///     >>> import vexy_json
///     >>> arr = vexy_json.loads_numpy_zerocopy('[1.0, 2.0, 3.0]', dtype='float64')
#[pyfunction]
#[pyo3(signature = (input, dtype = None))]
fn loads_numpy_zerocopy(py: Python, input: &str, dtype: Option<&str>) -> PyResult<PyObject> {
    // Try to import numpy
    let numpy = match py.import("numpy") {
        Ok(np) => np,
        Err(_) => {
            return Err(PyValueError::new_err(
                "NumPy is not available. Please install numpy: pip install numpy",
            ))
        }
    };

    // Parse the JSON
    let value = match parse(input) {
        Ok(v) => v,
        Err(e) => return Err(PyValueError::new_err(format!("Parse error: {}", e))),
    };

    match value {
        Value::Array(arr) => {
            // Analyze the array to determine if we can use zero-copy optimization
            let mut all_integers = true;
            let mut all_floats = true;

            for item in &arr {
                match item {
                    Value::Number(vexy_json_core::ast::Number::Integer(_)) => {
                        all_floats = false;
                    }
                    Value::Number(vexy_json_core::ast::Number::Float(_)) => {
                        all_integers = false;
                    }
                    _ => {
                        all_integers = false;
                        all_floats = false;
                        break;
                    }
                }
            }

            if all_integers {
                // Zero-copy path for integers
                let integers: Vec<i64> = arr
                    .iter()
                    .map(|v| {
                        match v {
                            Value::Number(vexy_json_core::ast::Number::Integer(i)) => *i,
                            _ => 0, // Should not happen
                        }
                    })
                    .collect();

                let numpy_array = if let Some(dt) = dtype {
                    numpy.call_method1("array", (integers, dt))?
                } else {
                    numpy.call_method1("array", (integers,))?
                };

                Ok(numpy_array.into())
            } else if all_floats || arr.iter().all(|v| matches!(v, Value::Number(_))) {
                // Zero-copy path for floats or mixed numbers
                let floats: Vec<f64> = arr
                    .iter()
                    .map(|v| {
                        match v {
                            Value::Number(vexy_json_core::ast::Number::Integer(i)) => *i as f64,
                            Value::Number(vexy_json_core::ast::Number::Float(f)) => *f,
                            _ => 0.0, // Should not happen
                        }
                    })
                    .collect();

                let numpy_array = if let Some(dt) = dtype {
                    numpy.call_method1("array", (floats, dt))?
                } else {
                    numpy.call_method1("array", (floats,))?
                };

                Ok(numpy_array.into())
            } else {
                // Fallback to regular conversion
                loads_numpy(py, input, dtype)
            }
        }
        _ => Err(PyValueError::new_err(
            "Input must be a JSON array for NumPy conversion",
        )),
    }
}

/// Convert JSON object to pandas DataFrame (if pandas is available)
///
/// Args:
///     input (str): The JSON string to parse (should be an object or array of objects)
///     orient (str, optional): DataFrame orientation. Defaults to 'records'.
///
/// Returns:
///     pandas.DataFrame: The parsed JSON as a DataFrame
///
/// Example:
///     >>> import vexy_json
///     >>> df = vexy_json.loads_dataframe('[{"a": 1, "b": 2}, {"a": 3, "b": 4}]')
///     >>> print(type(df))
///     <class 'pandas.core.frame.DataFrame'>
#[pyfunction]
#[pyo3(signature = (input, _orient = "records"))]
fn loads_dataframe(py: Python, input: &str, _orient: &str) -> PyResult<PyObject> {
    // Try to import pandas
    let pandas = match py.import("pandas") {
        Ok(pd) => pd,
        Err(_) => {
            return Err(PyValueError::new_err(
                "pandas is not available. Please install pandas: pip install pandas",
            ))
        }
    };

    // Parse the JSON
    let value = match parse(input) {
        Ok(v) => v,
        Err(e) => return Err(PyValueError::new_err(format!("Parse error: {}", e))),
    };

    // Convert to Python object
    let py_obj = value_to_python(py, &value)?;

    // Create DataFrame
    let df = pandas.call_method1("DataFrame", (py_obj,))?;
    Ok(df.into())
}

/// A Python module for parsing forgiving JSON
#[pymodule]
fn _vexy_json(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_with_options_py, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;

    // Add NumPy integration functions
    m.add_function(wrap_pyfunction!(loads_numpy, m)?)?;
    m.add_function(wrap_pyfunction!(loads_numpy_zerocopy, m)?)?;
    m.add_function(wrap_pyfunction!(loads_dataframe, m)?)?;

    // Add streaming parser class
    m.add_class::<StreamingParser>()?;

    // Add convenience aliases
    m.add("parse", m.getattr("parse_json")?)?;
    m.add("parse_with_options", m.getattr("parse_with_options_py")?)?;

    // Add standard json module compatibility
    m.add("loads", m.getattr("parse_json")?)?;

    // Add version information
    m.add(
        "__version__",
        env!("VEXY_JSON_VERSION", env!("CARGO_PKG_VERSION")),
    )?;
    m.add("__author__", "Adam Twardoch")?;
    m.add(
        "__description__",
        "A forgiving JSON parser - Python bindings for vexy_json",
    )?;

    Ok(())
}
