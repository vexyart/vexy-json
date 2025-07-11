use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pythonize::{depythonize, pythonize};
use serde_json::Value as JsonValue;
use vexy_json_core::{
    parse as core_parse, parse_with_options as core_parse_with_options, Parser, ParserOptions,
    RepairMode,
};

/// Parse error that can be raised in Python
#[pyclass]
struct ParseError {
    #[pyo3(get)]
    message: String,
    #[pyo3(get)]
    line: Option<usize>,
    #[pyo3(get)]
    column: Option<usize>,
}

#[pymethods]
impl ParseError {
    fn __str__(&self) -> String {
        if let (Some(line), Some(col)) = (self.line, self.column) {
            format!("Parse error at {}:{}: {}", line, col, self.message)
        } else {
            format!("Parse error: {}", self.message)
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParseError(message='{}', line={:?}, column={:?})",
            self.message, self.line, self.column
        )
    }
}

/// Repair information
#[pyclass]
#[derive(Clone)]
struct Repair {
    #[pyo3(get)]
    repair_type: String,
    #[pyo3(get)]
    position: usize,
    #[pyo3(get)]
    description: String,
}

#[pymethods]
impl Repair {
    fn __repr__(&self) -> String {
        format!(
            "Repair(type='{}', position={}, description='{}')",
            self.repair_type, self.position, self.description
        )
    }
}

/// Parser options for configuring vexy_json behavior
#[pyclass]
#[derive(Clone)]
struct Options {
    inner: ParserOptions,
}

#[pymethods]
impl Options {
    #[new]
    #[pyo3(signature = (
        allow_comments=true,
        allow_trailing_commas=true,
        allow_unquoted_keys=true,
        allow_single_quotes=true,
        implicit_top_level=true,
        newline_as_comma=true,
        max_depth=128,
        enable_repair=true,
        max_repairs=100,
        fast_repair=false,
        report_repairs=false
    ))]
    fn new(
        allow_comments: bool,
        allow_trailing_commas: bool,
        allow_unquoted_keys: bool,
        allow_single_quotes: bool,
        implicit_top_level: bool,
        newline_as_comma: bool,
        max_depth: u32,
        enable_repair: bool,
        max_repairs: u32,
        fast_repair: bool,
        report_repairs: bool,
    ) -> Self {
        let mut options = ParserOptions::default();
        options.allow_comments = allow_comments;
        options.allow_trailing_commas = allow_trailing_commas;
        options.allow_unquoted_keys = allow_unquoted_keys;
        options.allow_single_quotes = allow_single_quotes;
        options.implicit_top_level = implicit_top_level;
        options.newline_as_comma = newline_as_comma;
        options.max_depth = max_depth;

        if enable_repair {
            options.repair_mode = if fast_repair {
                RepairMode::Fast
            } else {
                RepairMode::Safe
            };
            options.max_repairs = max_repairs;
        } else {
            options.repair_mode = RepairMode::None;
        }

        options.track_path = report_repairs;

        Options { inner: options }
    }

    /// Create default options (all forgiving features enabled)
    #[staticmethod]
    fn default() -> Self {
        Options {
            inner: ParserOptions::default(),
        }
    }

    /// Create strict options (standard JSON only)
    #[staticmethod]
    fn strict() -> Self {
        Options {
            inner: ParserOptions::strict(),
        }
    }

    #[getter]
    fn allow_comments(&self) -> bool {
        self.inner.allow_comments
    }

    #[setter]
    fn set_allow_comments(&mut self, value: bool) {
        self.inner.allow_comments = value;
    }

    #[getter]
    fn allow_trailing_commas(&self) -> bool {
        self.inner.allow_trailing_commas
    }

    #[setter]
    fn set_allow_trailing_commas(&mut self, value: bool) {
        self.inner.allow_trailing_commas = value;
    }

    #[getter]
    fn allow_unquoted_keys(&self) -> bool {
        self.inner.allow_unquoted_keys
    }

    #[setter]
    fn set_allow_unquoted_keys(&mut self, value: bool) {
        self.inner.allow_unquoted_keys = value;
    }

    #[getter]
    fn allow_single_quotes(&self) -> bool {
        self.inner.allow_single_quotes
    }

    #[setter]
    fn set_allow_single_quotes(&mut self, value: bool) {
        self.inner.allow_single_quotes = value;
    }

    #[getter]
    fn implicit_top_level(&self) -> bool {
        self.inner.implicit_top_level
    }

    #[setter]
    fn set_implicit_top_level(&mut self, value: bool) {
        self.inner.implicit_top_level = value;
    }

    #[getter]
    fn newline_as_comma(&self) -> bool {
        self.inner.newline_as_comma
    }

    #[setter]
    fn set_newline_as_comma(&mut self, value: bool) {
        self.inner.newline_as_comma = value;
    }

    #[getter]
    fn max_depth(&self) -> u32 {
        self.inner.max_depth
    }

    #[setter]
    fn set_max_depth(&mut self, value: u32) {
        self.inner.max_depth = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "Options(allow_comments={}, allow_trailing_commas={}, allow_unquoted_keys={}, ...)",
            self.inner.allow_comments,
            self.inner.allow_trailing_commas,
            self.inner.allow_unquoted_keys
        )
    }
}

/// Result of parsing with repair information
#[pyclass]
struct ParseResult {
    #[pyo3(get)]
    data: PyObject,
    #[pyo3(get)]
    repairs: Vec<Repair>,
}

/// vexy_json parser instance
#[pyclass]
struct Parser {
    parser: vexy_json_core::Parser,
}

#[pymethods]
impl Parser {
    #[new]
    #[pyo3(signature = (options=None))]
    fn new(options: Option<Options>) -> Self {
        let opts = options.map(|o| o.inner).unwrap_or_default();
        Parser {
            parser: vexy_json_core::Parser::new(opts),
        }
    }

    /// Parse JSON string and return Python object
    fn parse(&self, py: Python, input: &str) -> PyResult<PyObject> {
        match self.parser.parse(input) {
            Ok(value) => json_to_python(py, &value),
            Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
        }
    }

    /// Parse JSON string and return result with repair information
    fn parse_detailed(&self, py: Python, input: &str) -> PyResult<ParseResult> {
        // For now, we'll just parse normally since repair tracking needs more work
        match self.parser.parse(input) {
            Ok(value) => {
                let data = json_to_python(py, &value)?;
                Ok(ParseResult {
                    data,
                    repairs: vec![],
                })
            }
            Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
        }
    }
}

/// Parse JSON string with default options
#[pyfunction]
#[pyo3(signature = (input))]
fn parse(py: Python, input: &str) -> PyResult<PyObject> {
    match core_parse(input) {
        Ok(value) => json_to_python(py, &value),
        Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
    }
}

/// Parse JSON string with custom options
#[pyfunction]
#[pyo3(signature = (input, options=None))]
fn parse_with_options(py: Python, input: &str, options: Option<Options>) -> PyResult<PyObject> {
    let opts = options.map(|o| o.inner).unwrap_or_default();
    match core_parse_with_options(input, opts) {
        Ok(value) => json_to_python(py, &value),
        Err(e) => Err(PyValueError::new_err(format!("Parse error: {}", e))),
    }
}

/// Dump Python object to JSON string
#[pyfunction]
#[pyo3(signature = (obj, indent=None, sort_keys=false))]
fn dumps(py: Python, obj: PyObject, indent: Option<usize>, sort_keys: bool) -> PyResult<String> {
    let value: JsonValue = depythonize(&obj.bind(py))
        .map_err(|e| PyTypeError::new_err(format!("Failed to convert to JSON: {}", e)))?;

    if let Some(indent) = indent {
        serde_json::to_string_pretty(&value)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))
    } else {
        serde_json::to_string(&value)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))
    }
}

/// Load JSON from file
#[pyfunction]
#[pyo3(signature = (filename, options=None))]
fn load(py: Python, filename: &str, options: Option<Options>) -> PyResult<PyObject> {
    let content = std::fs::read_to_string(filename)
        .map_err(|e| PyValueError::new_err(format!("Failed to read file: {}", e)))?;
    parse_with_options(py, &content, options)
}

/// Dump Python object to JSON file
#[pyfunction]
#[pyo3(signature = (obj, filename, indent=None, sort_keys=false))]
fn dump(
    py: Python,
    obj: PyObject,
    filename: &str,
    indent: Option<usize>,
    sort_keys: bool,
) -> PyResult<()> {
    let json_str = dumps(py, obj, indent, sort_keys)?;
    std::fs::write(filename, json_str)
        .map_err(|e| PyValueError::new_err(format!("Failed to write file: {}", e)))?;
    Ok(())
}

/// Get vexy_json version
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// Helper function to convert JSON values to Python objects
fn json_to_python(py: Python, value: &JsonValue) -> PyResult<PyObject> {
    pythonize(py, value).map_err(|e| PyValueError::new_err(format!("Conversion error: {}", e)))
}

/// vexy_json - A forgiving JSON parser for Python
#[pymodule]
fn vexy_json(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ParseError>()?;
    m.add_class::<Repair>()?;
    m.add_class::<Options>()?;
    m.add_class::<ParseResult>()?;
    m.add_class::<Parser>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
