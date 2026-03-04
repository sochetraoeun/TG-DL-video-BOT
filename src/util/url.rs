use linkify::{LinkFinder, LinkKind};

use crate::platform::matcher::{detect_platform, Platform};

pub struct ExtractedUrl {
    pub url: String,
    pub platform: Platform,
}

pub fn extract_supported_urls(text: &str) -> Vec<ExtractedUrl> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    finder
        .links(text)
        .filter_map(|link| {
            let url = link.as_str().to_string();
            detect_platform(&url).map(|platform| ExtractedUrl { url, platform })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_url() {
        let text = "Check this out https://www.youtube.com/watch?v=abc123";
        let urls = extract_supported_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].platform, Platform::YouTube);
    }

    #[test]
    fn test_extract_multiple_urls() {
        let text = "YouTube: https://youtu.be/abc and TikTok: https://www.tiktok.com/@user/video/1";
        let urls = extract_supported_urls(text);
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_no_supported_urls() {
        let text = "Check https://example.com for details";
        let urls = extract_supported_urls(text);
        assert!(urls.is_empty());
    }

    #[test]
    fn test_plain_text_no_urls() {
        let text = "Hello, this is a plain message";
        let urls = extract_supported_urls(text);
        assert!(urls.is_empty());
    }
}
