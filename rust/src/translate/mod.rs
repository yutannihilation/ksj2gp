use std::sync::LazyLock;

use regex::Regex;

pub(crate) fn extract_ksj_id(filename: &str) -> Result<&str, crate::Ksj2GpError> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[A-Z][0-9]{2}[a-z]?[0-9]?(?:-[a-z])?(?:-[cu])?").unwrap());

    // Handle special cases first

    // A19s-a is a variant of A19s
    if filename.contains("A19s-a") {
        return Ok("A19s");
    }

    // _ to -
    if filename.contains("A18s_a") {
        return Ok("A18s-a");
    }

    match RE.captures(filename) {
        Some(c) => Ok(c.extract::<0>().0),
        None => Err(format!("Failed to detect KSJ id from filename: {filename}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::extract_ksj_id;

    #[test]
    fn test_extract_ksj_id() {
        assert_eq!(extract_ksj_id("A03-03_SYUTO-tky_GML.zip").unwrap(), "A03");
        assert_eq!(extract_ksj_id("S05-a-10_KINKI_GML.zip").unwrap(), "S05-a");
        assert_eq!(
            extract_ksj_id("A30a5-11_5338-jgd_GML.zip").unwrap(),
            "A30a5"
        );
        assert_eq!(
            extract_ksj_id("L03-b-c-16_5440_GML.zip").unwrap(),
            "L03-b-c"
        );
        assert_eq!(
            extract_ksj_id("L03-b-16_3623-tky_GML.zip").unwrap(),
            "L03-b"
        );
        assert_eq!(extract_ksj_id("A19s-a-10_28_GML.zip").unwrap(), "A19s");
    }
}
