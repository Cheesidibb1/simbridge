// Byte-related utilities

/// Byte utilities
pub struct ByteUtils;

impl ByteUtils {
    /// Format bytes in human-readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// Convert human-readable string to bytes
    pub fn parse_bytes(s: &str) -> Result<u64, String> {
        let s = s.trim().to_uppercase();
        let (num_str, unit) = if let Some(pos) = s.find(|c: char| !c.is_numeric() && c != '.') {
            (&s[..pos], s[pos..].trim())
        } else {
            (&s[..], "B")
        };

        let num: f64 = num_str.parse()
            .map_err(|_| format!("Invalid number: {}", num_str))?;

        let multiplier = match unit {
            "B" => 1.0,
            "KB" => 1024.0,
            "MB" => 1024.0 * 1024.0,
            "GB" => 1024.0 * 1024.0 * 1024.0,
            "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => return Err(format!("Unknown unit: {}", unit)),
        };

        Ok((num * multiplier) as u64)
    }

    /// Convert bits to bytes
    pub fn bits_to_bytes(bits: u64) -> u64 {
        bits / 8
    }

    /// Convert bytes to bits
    pub fn bytes_to_bits(bytes: u64) -> u64 {
        bytes * 8
    }

    /// Calculate bits per second from bytes and duration
    pub fn calculate_bps(bytes: u64, duration_secs: f64) -> f64 {
        if duration_secs == 0.0 {
            0.0
        } else {
            (bytes * 8) as f64 / duration_secs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(ByteUtils::format_bytes(500), "500 B");
        assert_eq!(ByteUtils::format_bytes(1536), "1.50 KB");
        assert_eq!(ByteUtils::format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_parse_bytes() {
        assert_eq!(ByteUtils::parse_bytes("500 B").unwrap(), 500);
        assert_eq!(ByteUtils::parse_bytes("1.5 KB").unwrap(), 1536);
        assert_eq!(ByteUtils::parse_bytes("1 MB").unwrap(), 1048576);
    }

    #[test]
    fn test_calculate_bps() {
        assert_eq!(ByteUtils::calculate_bps(1000, 1.0), 8000.0);
        assert_eq!(ByteUtils::calculate_bps(1000, 0.0), 0.0);
    }
}
