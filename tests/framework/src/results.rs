use crate::options::{Approximations, ImageComparison};
use image::RgbaImage;

#[derive(Default)]
pub struct TestResults {
    pub results: Vec<TestResult>,
}

impl TestResults {
    pub fn success(&self) -> bool {
        self.results.iter().all(|img| img.success())
    }

    pub fn add(&mut self, result: impl Into<TestResult>) {
        self.results.push(result.into());
    }
}

pub enum TestResult {
    Trace(TraceComparisonResult),
    Approximation(NumberApproximationResult),
    Image(ImageComparisonResult),
    InvalidTest(String),
}

impl TestResult {
    pub fn success(&self) -> bool {
        match self {
            TestResult::Trace(result) => result.success(),
            TestResult::Approximation(result) => result.success(),
            TestResult::Image(result) => result.success(),
            TestResult::InvalidTest(_) => false,
        }
    }
}

impl From<String> for TestResult {
    fn from(value: String) -> Self {
        TestResult::InvalidTest(value)
    }
}

pub struct TraceComparisonResult {
    pub expected: String,
    pub actual: String,
}

impl TraceComparisonResult {
    pub fn success(&self) -> bool {
        self.expected == self.actual
    }
}

pub struct NumberApproximationResult {
    pub line: usize,
    pub expected: f64,
    pub actual: f64,
    pub options: Approximations,
    pub group: Option<u32>,
    pub surrounding_text: Option<TraceComparisonResult>,
}

impl NumberApproximationResult {
    pub fn success(&self) -> bool {
        if let Some(surrounding_text) = &self.surrounding_text {
            if !surrounding_text.success() {
                return false;
            }
        }
        // NaNs should be able to pass in an approx test.
        if self.actual.is_nan() && self.expected.is_nan() {
            return true;
        }
        self.options.compare(self.actual, self.expected)
    }
}

pub struct ImageComparisonResult {
    pub name: String,
    pub expected: RgbaImage,
    pub actual: RgbaImage,
    pub difference_color: Option<RgbaImage>,
    pub difference_alpha: Option<RgbaImage>,
    pub options: ImageComparison,
}

impl ImageComparisonResult {
    pub fn success(&self) -> bool {
        self.expected == self.actual
    }
}
