use dht_crawler::FileInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Video = 1,
    Audio = 2,
    Doc = 3,
    App = 4,
    Other = 5,
}

#[derive(Debug, Clone, Copy)]
pub struct CategoryMeta {
    pub key: &'static str,
    pub label: &'static str,
    pub code: i16,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Video,
        Category::Audio,
        Category::Doc,
        Category::App,
        Category::Other,
    ];

    pub const fn code(self) -> i16 {
        self as i16
    }

    pub const fn key(self) -> &'static str {
        match self {
            Category::Video => "video",
            Category::Audio => "audio",
            Category::Doc => "doc",
            Category::App => "app",
            Category::Other => "other",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Category::Video => "Video",
            Category::Audio => "Audio",
            Category::Doc => "Doc",
            Category::App => "App",
            Category::Other => "Other",
        }
    }

    pub fn from_code(code: i16) -> Self {
        match code {
            1 => Category::Video,
            2 => Category::Audio,
            3 => Category::Doc,
            4 => Category::App,
            _ => Category::Other,
        }
    }

    pub fn parse_filter(raw: Option<&str>) -> Result<Option<Self>, String> {
        match raw {
            None => Ok(None),
            Some(value) => {
                let normalized = value.trim().to_ascii_lowercase();
                match normalized.as_str() {
                    "" | "all" => Ok(None),
                    "video" => Ok(Some(Self::Video)),
                    "audio" => Ok(Some(Self::Audio)),
                    "doc" => Ok(Some(Self::Doc)),
                    "app" => Ok(Some(Self::App)),
                    "other" => Ok(Some(Self::Other)),
                    _ => Err(format!("unsupported category: {value}")),
                }
            }
        }
    }

    pub fn classify(name: &str, files: &[FileInfo]) -> Self {
        let mut scores = [0u64; 5];

        for file in files {
            let ext = extension_of(&file.path);
            let bucket = classify_extension(ext.as_deref()).unwrap_or(Self::Other);
            let idx = (bucket.code() - 1) as usize;
            scores[idx] = scores[idx].saturating_add(file.size.max(1));
        }

        if files.is_empty() {
            let ext = extension_of(name);
            return classify_extension(ext.as_deref()).unwrap_or(Self::Other);
        }

        let mut max_idx = 4usize;
        let mut max_score = 0u64;
        for (idx, score) in scores.iter().enumerate() {
            if *score > max_score {
                max_score = *score;
                max_idx = idx;
            }
        }

        match max_idx {
            0 => Self::Video,
            1 => Self::Audio,
            2 => Self::Doc,
            3 => Self::App,
            _ => Self::Other,
        }
    }
}

pub fn all_category_meta() -> Vec<CategoryMeta> {
    let mut out = Vec::with_capacity(6);
    out.push(CategoryMeta {
        key: "all",
        label: "All",
        code: 0,
    });
    for category in Category::ALL {
        out.push(CategoryMeta {
            key: category.key(),
            label: category.label(),
            code: category.code(),
        });
    }
    out
}

fn extension_of(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .and_then(|ext| if ext.is_empty() { None } else { Some(ext) })
}

fn classify_extension(ext: Option<&str>) -> Option<Category> {
    let ext = ext?;

    if matches!(
        ext,
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mpeg" | "mpg" | "ts"
    ) {
        return Some(Category::Video);
    }

    if matches!(ext, "mp3" | "flac" | "aac" | "m4a" | "wav" | "ogg" | "opus") {
        return Some(Category::Audio);
    }

    if matches!(
        ext,
        "pdf"
            | "epub"
            | "mobi"
            | "azw"
            | "doc"
            | "docx"
            | "ppt"
            | "pptx"
            | "xls"
            | "xlsx"
            | "txt"
            | "md"
    ) {
        return Some(Category::Doc);
    }

    if matches!(
        ext,
        "exe"
            | "msi"
            | "dmg"
            | "pkg"
            | "apk"
            | "ipa"
            | "iso"
            | "zip"
            | "rar"
            | "7z"
            | "tar"
            | "gz"
            | "bz2"
            | "xz"
            | "deb"
            | "rpm"
    ) {
        return Some(Category::App);
    }

    Some(Category::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_category() {
        assert!(matches!(Category::parse_filter(Some("all")), Ok(None)));
    }

    #[test]
    fn parse_named_category() {
        assert!(matches!(
            Category::parse_filter(Some("video")),
            Ok(Some(Category::Video))
        ));
    }
}
