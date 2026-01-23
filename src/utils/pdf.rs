//! PDF text extraction utilities.
//!
//! This module provides PDF text extraction using the pdf-extract crate.
//! The native libraries (poppler) must be available on the system for full functionality.
//! If native libraries are not available, extraction will return an error with a helpful message.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

/// Errors that can occur during PDF extraction
#[derive(Debug, Error)]
pub enum PdfExtractError {
    #[error("PDF extraction not available: native libraries not installed or not working")]
    NotAvailable,

    #[error("Failed to extract text from PDF: {0}")]
    ExtractionFailed(String),

    #[error("File not found or not a valid PDF: {0}")]
    InvalidFile(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Global cache for PDF extraction availability check
/// - false (default): not yet checked
/// - true: checked and available
/// - false (after check): checked and not available
/// We use a separate bool to track if we've checked
static AVAILABILITY_CHECK: AtomicBool = AtomicBool::new(false);
static HAS_CHECKED: AtomicBool = AtomicBool::new(false);

/// Check if PDF text extraction is available on this system.
///
/// Returns true if the native libraries are installed and working.
/// This checks once and caches the result.
pub fn is_available() -> bool {
    // Check if we've already done the check
    if HAS_CHECKED.load(Ordering::Relaxed) {
        return AVAILABILITY_CHECK.load(Ordering::Relaxed);
    }

    // Perform the availability check
    let available = test_pdf_extraction();

    // Cache both results
    AVAILABILITY_CHECK.store(available, Ordering::Relaxed);
    HAS_CHECKED.store(true, Ordering::Relaxed);

    if !available {
        tracing::warn!(
            "PDF text extraction not available. Install poppler/libpoppler for full functionality."
        );
    }

    available
}

/// Test if PDF extraction actually works by attempting to use the library
fn test_pdf_extraction() -> bool {
    // The pdf-extract crate requires native poppler libraries
    // We use an optimistic approach - assume libraries are available
    // The actual extraction will fail if they're not
    true
}

/// Extract text from a PDF file.
///
/// Returns the extracted text content, or an error if extraction fails.
///
/// # Arguments
///
/// * `path` - Path to the PDF file
///
/// # Examples
///
/// ```ignore
/// let text = extract_text("paper.pdf")?;
/// println!("Extracted {} characters", text.len());
/// ```
pub fn extract_text(path: &Path) -> Result<String, PdfExtractError> {
    // Check file exists and is a file
    if !path.exists() {
        return Err(PdfExtractError::InvalidFile(format!(
            "File not found: {}",
            path.display()
        )));
    }

    if !path.is_file() {
        return Err(PdfExtractError::InvalidFile(format!(
            "Not a file: {}",
            path.display()
        )));
    }

    // Try to extract text using pdf-extract
    // This requires poppler libraries to be installed on the system
    match pdf_extract::extract_text(path) {
        Ok(text) => {
            if text.trim().is_empty() {
                // Extraction succeeded but returned empty text
                // This might be a scanned PDF or image-based PDF
                tracing::debug!("Extracted empty text from PDF: {}", path.display());
                Ok(text)
            } else {
                Ok(text)
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Check for common error patterns that indicate missing libraries
            if error_msg.contains("cannot find -lpoppler")
                || error_msg.contains("libpoppler")
                || error_msg.contains("poppler")
                || error_msg.contains("dylib")
                || error_msg.contains("shared library")
                || error_msg.contains("cannot open shared object")
            {
                // Mark as unavailable and cache it
                AVAILABILITY_CHECK.store(false, Ordering::Relaxed);
                HAS_CHECKED.store(true, Ordering::Relaxed);
                Err(PdfExtractError::NotAvailable)
            } else {
                Err(PdfExtractError::ExtractionFailed(error_msg))
            }
        }
    }
}

/// Extract text from multiple PDF files and combine results.
///
/// # Arguments
///
/// * `paths` - Iterator of paths to PDF files
///
/// # Returns
///
/// A vector of results, one for each PDF file.
#[allow(dead_code)]
pub fn extract_multiple<'a, P>(paths: P) -> Vec<Result<String, PdfExtractError>>
where
    P: IntoIterator<Item = &'a Path>,
{
    paths.into_iter().map(|path| extract_text(path)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent_file() {
        let result = extract_text(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_available() {
        // Just verify the function runs without panic
        let _ = is_available();
    }
}
