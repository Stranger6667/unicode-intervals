use pyo3::{exceptions::PyValueError, prelude::*, types::PyTuple, wrap_pyfunction};
use unicode_intervals_core::{UnicodeCategory, UnicodeCategorySet, UnicodeVersion as CoreVersion};

/// A bundled Unicode version, e.g. ``UnicodeVersion("16.0.0")``.
#[pyclass(
    eq,
    frozen,
    from_py_object,
    name = "UnicodeVersion",
    module = "unicode_intervals"
)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyUnicodeVersion {
    V9_0_0,
    V10_0_0,
    V11_0_0,
    V12_0_0,
    V12_1_0,
    V13_0_0,
    V14_0_0,
    V15_0_0,
    V15_1_0,
    V16_0_0,
    V17_0_0,
}

impl From<PyUnicodeVersion> for CoreVersion {
    fn from(value: PyUnicodeVersion) -> Self {
        match value {
            PyUnicodeVersion::V9_0_0 => CoreVersion::V9_0_0,
            PyUnicodeVersion::V10_0_0 => CoreVersion::V10_0_0,
            PyUnicodeVersion::V11_0_0 => CoreVersion::V11_0_0,
            PyUnicodeVersion::V12_0_0 => CoreVersion::V12_0_0,
            PyUnicodeVersion::V12_1_0 => CoreVersion::V12_1_0,
            PyUnicodeVersion::V13_0_0 => CoreVersion::V13_0_0,
            PyUnicodeVersion::V14_0_0 => CoreVersion::V14_0_0,
            PyUnicodeVersion::V15_0_0 => CoreVersion::V15_0_0,
            PyUnicodeVersion::V15_1_0 => CoreVersion::V15_1_0,
            PyUnicodeVersion::V16_0_0 => CoreVersion::V16_0_0,
            PyUnicodeVersion::V17_0_0 => CoreVersion::V17_0_0,
        }
    }
}

impl From<CoreVersion> for PyUnicodeVersion {
    fn from(value: CoreVersion) -> Self {
        match value {
            CoreVersion::V9_0_0 => PyUnicodeVersion::V9_0_0,
            CoreVersion::V10_0_0 => PyUnicodeVersion::V10_0_0,
            CoreVersion::V11_0_0 => PyUnicodeVersion::V11_0_0,
            CoreVersion::V12_0_0 => PyUnicodeVersion::V12_0_0,
            CoreVersion::V12_1_0 => PyUnicodeVersion::V12_1_0,
            CoreVersion::V13_0_0 => PyUnicodeVersion::V13_0_0,
            CoreVersion::V14_0_0 => PyUnicodeVersion::V14_0_0,
            CoreVersion::V15_0_0 => PyUnicodeVersion::V15_0_0,
            CoreVersion::V15_1_0 => PyUnicodeVersion::V15_1_0,
            CoreVersion::V16_0_0 => PyUnicodeVersion::V16_0_0,
            CoreVersion::V17_0_0 => PyUnicodeVersion::V17_0_0,
            // `CoreVersion` is non-exhaustive; fail loud instead of mismapping a new version.
            _ => unreachable!("core Unicode version not mirrored in the binding"),
        }
    }
}

#[pymethods]
impl PyUnicodeVersion {
    /// Parse a version string such as ``"16.0.0"``.
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        value
            .parse::<CoreVersion>()
            .map(Into::into)
            .map_err(|_| PyValueError::new_err(format!("Unknown Unicode version: {value}")))
    }
    /// The latest bundled Unicode version.
    #[staticmethod]
    fn latest() -> Self {
        CoreVersion::latest().into()
    }
    fn __str__(&self) -> &'static str {
        CoreVersion::from(*self).as_str()
    }
    fn __repr__(&self) -> String {
        format!("UnicodeVersion(\"{}\")", CoreVersion::from(*self).as_str())
    }
}

fn parse_categories(names: Option<Vec<String>>) -> PyResult<Option<UnicodeCategorySet>> {
    match names {
        None => Ok(None),
        Some(list) => {
            let mut set = UnicodeCategorySet::new();
            for name in list {
                let category: UnicodeCategory = name
                    .parse()
                    .map_err(|_| PyValueError::new_err(format!("Unknown category: {name}")))?;
                set |= category;
            }
            Ok(Some(set))
        }
    }
}

/// Return a tuple of inclusive ``(start, end)`` code point intervals matching the criteria.
///
/// ``categories``/``exclude_categories`` take abbreviations like ``"Lu"``;
/// ``include_characters``/``exclude_characters`` add or drop individual code points;
/// ``version`` defaults to the latest bundled Unicode version.
#[pyfunction]
#[pyo3(signature = (*, categories=None, exclude_categories=None, min_codepoint=None,
                    max_codepoint=None, include_characters="", exclude_characters="",
                    version=None))]
#[allow(clippy::too_many_arguments)]
fn query(
    py: Python<'_>,
    categories: Option<Vec<String>>,
    exclude_categories: Option<Vec<String>>,
    min_codepoint: Option<u32>,
    max_codepoint: Option<u32>,
    include_characters: &str,
    exclude_characters: &str,
    version: Option<PyUnicodeVersion>,
) -> PyResult<Py<PyTuple>> {
    let core_version: CoreVersion = version.map_or_else(CoreVersion::latest, Into::into);
    let include = parse_categories(categories)?;
    let exclude = parse_categories(exclude_categories)?.unwrap_or_default();
    let mut builder = core_version
        .query()
        .include_categories(include)
        .exclude_categories(exclude)
        .include_characters(include_characters)
        .exclude_characters(exclude_characters);
    if let Some(lo) = min_codepoint {
        builder = builder.min_codepoint(lo);
    }
    if let Some(hi) = max_codepoint {
        builder = builder.max_codepoint(hi);
    }
    let intervals = builder
        .intervals()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyTuple::new(py, intervals)?.into())
}

/// Unicode category abbreviations in normalised order (control and surrogate last).
#[pyfunction]
#[pyo3(signature = (version=None))]
fn categories(py: Python<'_>, version: Option<PyUnicodeVersion>) -> PyResult<Py<PyTuple>> {
    let core_version: CoreVersion = version.map_or_else(CoreVersion::latest, Into::into);
    let names: Vec<String> = core_version
        .normalized_categories()
        .iter()
        .map(ToString::to_string)
        .collect();
    Ok(PyTuple::new(py, names)?.into())
}

/// Expand major classes (e.g. ``"N"`` -> ``"Nd"``, ``"Nl"``, ``"No"``) and validate category names.
#[pyfunction]
#[pyo3(signature = (categories, version=None))]
fn as_general_categories(
    py: Python<'_>,
    categories: Vec<String>,
    version: Option<PyUnicodeVersion>,
) -> PyResult<Py<PyTuple>> {
    let core_version: CoreVersion = version.map_or_else(CoreVersion::latest, Into::into);
    let expanded = unicode_intervals_core::as_general_categories(&categories, core_version)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let names: Vec<String> = expanded.iter().map(ToString::to_string).collect();
    Ok(PyTuple::new(py, names)?.into())
}

/// Search Unicode code point intervals by category, codepoint range, and characters.
#[pymodule]
fn unicode_intervals(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyUnicodeVersion>()?;
    let versions: Vec<PyUnicodeVersion> = CoreVersion::ALL.iter().map(|v| (*v).into()).collect();
    module.add("available_versions", PyTuple::new(py, versions)?)?;
    module.add_wrapped(wrap_pyfunction!(query))?;
    module.add_wrapped(wrap_pyfunction!(categories))?;
    module.add_wrapped(wrap_pyfunction!(as_general_categories))?;
    Ok(())
}
