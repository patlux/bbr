//! Blueprint D15: Safari's `Bookmarks.plist` (binary or XML) read through
//! Foundation's property-list decoding — the leaves' titles and URLs only.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSDictionary, NSString, NSURL};

fn string(dictionary: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<String> {
    let value = dictionary.objectForKey(&NSString::from_str(key))?;
    value.downcast_ref::<NSString>().map(|s| s.to_string())
}

fn walk(node: &NSDictionary<NSString, AnyObject>, out: &mut Vec<(String, String)>) {
    if string(node, "WebBookmarkType").as_deref() == Some("WebBookmarkTypeLeaf") {
        let url = string(node, "URLString").unwrap_or_default();
        let title = node
            .objectForKey(&NSString::from_str("URIDictionary"))
            .and_then(|uri| {
                uri.downcast::<NSDictionary>().ok().map(|uri| unsafe {
                    Retained::cast_unchecked::<NSDictionary<NSString, AnyObject>>(uri)
                })
            })
            .and_then(|uri| string(&uri, "title"))
            .unwrap_or_default();
        out.push((title, url));
        return;
    }
    let Some(children) = node.objectForKey(&NSString::from_str("Children")) else {
        return;
    };
    let Ok(children) = children.downcast::<NSArray>() else {
        return;
    };
    for child in children.iter() {
        if let Ok(child) = child.downcast::<NSDictionary>() {
            // SAFETY: property-list dictionaries have string keys.
            let child =
                unsafe { Retained::cast_unchecked::<NSDictionary<NSString, AnyObject>>(child) };
            walk(&child, out);
        }
    }
}

/// The `(title, url)` of every bookmark in the plist at `path`; an error
/// when the file cannot be read (missing, or protected by macOS).
pub fn read(path: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    std::fs::metadata(path).map_err(|e| e.to_string())?;
    std::fs::File::open(path).map_err(|e| e.to_string())?;
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    // SAFETY: a file URL; Foundation decodes binary and XML plists.
    #[allow(deprecated)]
    let root = unsafe { NSDictionary::<NSString, AnyObject>::dictionaryWithContentsOfURL(&url) }
        .ok_or_else(|| "not a property list".to_string())?;
    let mut out = Vec::new();
    walk(&root, &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn xml_bookmarks_are_read_depth_first() {
        let dir = std::env::temp_dir().join(format!("bbr-plist-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Bookmarks.plist");
        std::fs::write(
            &path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>WebBookmarkType</key><string>WebBookmarkTypeList</string>
<key>Children</key><array>
  <dict><key>WebBookmarkType</key><string>WebBookmarkTypeList</string><key>Title</key><string>BookmarksBar</string>
    <key>Children</key><array>
      <dict><key>WebBookmarkType</key><string>WebBookmarkTypeLeaf</string><key>URLString</key><string>https://webkit.org/</string>
        <key>URIDictionary</key><dict><key>title</key><string>WebKit</string></dict></dict>
    </array></dict>
  <dict><key>WebBookmarkType</key><string>WebBookmarkTypeLeaf</string><key>URLString</key><string>https://apple.com/</string>
    <key>URIDictionary</key><dict><key>title</key><string>Apple</string></dict></dict>
</array></dict></plist>"#,
        )
        .unwrap();
        let bookmarks = super::read(&path).expect("plist");
        assert_eq!(
            bookmarks,
            [
                ("WebKit".to_string(), "https://webkit.org/".to_string()),
                ("Apple".to_string(), "https://apple.com/".to_string())
            ]
        );
        assert!(super::read(&dir.join("missing.plist")).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
