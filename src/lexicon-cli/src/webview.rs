use once_cell::sync::Lazy;

#[allow(dead_code)]
use std::collections::HashMap;
use std::sync::Mutex;
// use webview::{Content, WebView};

static WEBVIEW_STATE: Lazy<Mutex<WebViewState>> = Lazy::new(|| Mutex::new(WebViewState::default()));

#[derive(Default)]
pub struct WebViewState {
    pub windows: Vec<WebViewWindow>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct WebViewWindow {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub html: String,
    pub scripts: Vec<String>,
    pub events: HashMap<String, String>,
}


pub fn register_window(window: WebViewWindow) {
    if let Ok(mut state) = WEBVIEW_STATE.lock() {
        state.windows.push(window);
    }
}

pub fn windows() -> Vec<WebViewWindow> {
    WEBVIEW_STATE
        .lock()
        .map(|state| state.windows.clone())
        .unwrap_or_default()
}

pub fn parse_webview_code(source: &str) -> Vec<WebViewWindow> {
    let mut windows = Vec::new();
    let mut current_window: Option<WebViewWindow> = None;

    let mut html_content = String::new();
    let mut body_started = false;
    let mut script_started = false;
    let mut script_content = String::new();

    for line in source.lines() {
        let line = line.trim();

        if line.contains("WebView::create") || line.contains("let webview = WebView") {
            if let Some(win) = current_window.take() {
                if !html_content.is_empty() || !win.scripts.is_empty() {
                    windows.push(win);
                }
            }
            let title = extract_title_from_line(line);
            let (width, height) = extract_size_u32(line);
            current_window = Some(WebViewWindow {
                title,
                width,
                height,
                html: String::new(),
                scripts: Vec::new(),
                events: HashMap::new(),
            });
            html_content = String::new();
            body_started = false;
            script_started = false;
            script_content = String::new();
        }

        if let Some(ref mut win) = current_window {
            if line.contains("<html") || line.contains("<!DOCTYPE") {
                html_content = format!("{}\n", line);
            } else if line.contains("<body") {
                body_started = true;
                html_content.push_str(&format!("{}\n", line));
            } else if body_started && !script_started {
                if line.contains("<script") {
                    script_started = true;
                } else if line.contains("</body>") || line.contains("</html>") {
                    body_started = false;
                    win.html = html_content.clone();
                    html_content.clear();
                } else {
                    html_content.push_str(&format!("{}\n", line));
                }
            } else if script_started {
                if line.contains("</script>") {
                    script_started = false;
                    win.scripts.push(script_content.clone());
                    script_content.clear();
                } else {
                    script_content.push_str(&format!("{}\n", line));
                }
            }

            if line.contains("setTitle(") || line.contains("title:") {
                win.title = extract_quoted_string(line);
            }
            if line.contains("setSize(") || line.contains("width:") || line.contains("height:") {
                if let Some((w, h)) = extract_size_u32_opt(line) {
                    win.width = w;
                    win.height = h;
                }
            }
        }
    }

    if let Some(mut win) = current_window {
        if !html_content.is_empty() {
            win.html.push_str(&html_content);
        }
        if !win.scripts.is_empty() || !win.html.is_empty() {
            windows.push(win);
        }
    }

    windows
}

fn extract_title_from_line(line: &str) -> String {
    if let Some(start) = line.find("title:") {
        let rest = &line[start + 6..];
        extract_quoted_string(rest)
    } else {
        "Lexicon WebView App".to_string()
    }
}

fn extract_quoted_string(s: &str) -> String {
    let s = s.trim();
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}

fn extract_size_u32(line: &str) -> (u32, u32) {
    let nums: Vec<u32> = line
        .split(|c: char| !c.is_numeric())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    if nums.len() >= 2 {
        (nums[0], nums[1])
    } else if nums.len() == 1 {
        (nums[0], 600)
    } else {
        (800, 600)
    }
}

fn extract_size_u32_opt(line: &str) -> Option<(u32, u32)> {
    let nums: Vec<u32> = line
        .split(|c: char| !c.is_numeric())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    if nums.len() >= 2 {
        Some((nums[0], nums[1]))
    } else if nums.len() == 1 {
        Some((nums[0], 600))
    } else {
        Some((800, 600))
    }
}

pub fn run_webview(source: &str) {
    let windows = parse_webview_code(source);

    if let Ok(mut state) = WEBVIEW_STATE.lock() {
        state.windows = windows.clone();
    }

    if windows.is_empty() {
        run_webview_window(
            "Lexicon WebView App",
            800,
            600,
            "<html><body><h1>Lexicon</h1></body></html>",
        );

        return;
    }

    for window in windows {
        let html = if window.html.is_empty() {
            generate_default_html(&window.title)
        } else {
            window.html
        };

        run_webview_window(
            &window.title,
            window.width,
            window.height,
            &html,
        );
    }
}

fn generate_default_html(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }}
        .container {{
            text-align: center;
            padding: 40px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
    </div>
</body>
</html>"#,
        title, title
    )
}

fn run_webview_window(title: &str, width: u32, height: u32, html: &str) {
    println!("Starting WebView: {} ({}x{})", title, width, height);

    let content = webview::Content::Html(html);
    let webview = webview::WebView::new(title, content, width as i32, height as i32, true, false);

    match webview {
        Ok(mut wv) => loop {
            wv.loop_once(true);
        },
        Err(e) => {
            eprintln!("Failed to create WebView: {:?}", e);
        }
    }
}
