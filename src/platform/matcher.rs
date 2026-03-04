use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    YouTube,
    TikTok,
    Instagram,
    Facebook,
}

impl Platform {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::YouTube => "YouTube",
            Self::TikTok => "TikTok",
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook",
        }
    }
}

static YOUTUBE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:https?://)?(?:www\.)?(?:youtube\.com|youtu\.be|m\.youtube\.com)/").unwrap()
});

static TIKTOK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:https?://)?(?:www\.)?(?:tiktok\.com|vm\.tiktok\.com)/").unwrap()
});

static INSTAGRAM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:https?://)?(?:www\.)?(?:instagram\.com|instagr\.am)/").unwrap()
});

static FACEBOOK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:https?://)?(?:www\.)?(?:facebook\.com|fb\.watch|fb\.com|m\.facebook\.com)/",
    )
    .unwrap()
});

pub fn detect_platform(url: &str) -> Option<Platform> {
    if YOUTUBE_RE.is_match(url) {
        Some(Platform::YouTube)
    } else if TIKTOK_RE.is_match(url) {
        Some(Platform::TikTok)
    } else if INSTAGRAM_RE.is_match(url) {
        Some(Platform::Instagram)
    } else if FACEBOOK_RE.is_match(url) {
        Some(Platform::Facebook)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_urls() {
        assert_eq!(
            detect_platform("https://www.youtube.com/watch?v=abc123"),
            Some(Platform::YouTube)
        );
        assert_eq!(
            detect_platform("https://youtu.be/abc123"),
            Some(Platform::YouTube)
        );
        assert_eq!(
            detect_platform("https://m.youtube.com/watch?v=abc123"),
            Some(Platform::YouTube)
        );
    }

    #[test]
    fn test_tiktok_urls() {
        assert_eq!(
            detect_platform("https://www.tiktok.com/@user/video/123"),
            Some(Platform::TikTok)
        );
        assert_eq!(
            detect_platform("https://vm.tiktok.com/ZMxx/"),
            Some(Platform::TikTok)
        );
    }

    #[test]
    fn test_instagram_urls() {
        assert_eq!(
            detect_platform("https://www.instagram.com/reel/abc123/"),
            Some(Platform::Instagram)
        );
        assert_eq!(
            detect_platform("https://www.instagram.com/p/abc123/"),
            Some(Platform::Instagram)
        );
    }

    #[test]
    fn test_facebook_urls() {
        assert_eq!(
            detect_platform("https://www.facebook.com/watch/?v=123"),
            Some(Platform::Facebook)
        );
        assert_eq!(
            detect_platform("https://fb.watch/abc123/"),
            Some(Platform::Facebook)
        );
    }

    #[test]
    fn test_unknown_url() {
        assert_eq!(detect_platform("https://example.com/video"), None);
    }
}
