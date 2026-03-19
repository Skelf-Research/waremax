//! Embedded static assets for the web UI
//!
//! Uses rust-embed to bundle the frontend into the binary.

use rust_embed::RustEmbed;

/// Embedded frontend assets
///
/// In development, files are loaded from disk.
/// In release, files are embedded in the binary.
#[derive(RustEmbed)]
#[folder = "frontend/dist"]
#[prefix = ""]
pub struct Assets;

impl Assets {
    /// Get a file by path
    pub fn get_file(path: &str) -> Option<rust_embed::EmbeddedFile> {
        Self::get(path)
    }

    /// Check if a file exists
    pub fn exists(path: &str) -> bool {
        Self::get(path).is_some()
    }

    /// Get the MIME type for a file
    pub fn mime_type(path: &str) -> &'static str {
        mime_guess::from_path(path)
            .first_raw()
            .unwrap_or("application/octet-stream")
    }

    /// List all embedded files
    pub fn list_files() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
        Self::iter()
    }
}

/// Get the index.html content (for SPA routing fallback)
pub fn get_index_html() -> Option<rust_embed::EmbeddedFile> {
    Assets::get("index.html")
}

/// Fallback HTML for when frontend is not built
pub const FALLBACK_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Waremax Simulation UI</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            max-width: 600px;
            text-align: center;
        }
        h1 {
            font-size: 2.5rem;
            margin-bottom: 1rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        p {
            color: #aaa;
            line-height: 1.6;
            margin-bottom: 1rem;
        }
        code {
            background: #16213e;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.9em;
            color: #667eea;
        }
        .status {
            margin-top: 2rem;
            padding: 1rem;
            background: #16213e;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }
        .api-check {
            margin-top: 1rem;
            font-size: 0.9em;
        }
        .api-check.success { color: #4ade80; }
        .api-check.error { color: #f87171; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Waremax Simulation</h1>
        <p>
            The web UI frontend has not been built yet.
            To use the interactive visualization, build the frontend first:
        </p>
        <div class="status">
            <p><strong>Build the frontend:</strong></p>
            <p><code>cd crates/waremax-ui/frontend && npm install && npm run build</code></p>
        </div>
        <div class="api-check" id="api-status">Checking API status...</div>
    </div>
    <script>
        fetch('/api/presets')
            .then(r => r.json())
            .then(data => {
                document.getElementById('api-status').textContent =
                    'API is running. ' + data.length + ' presets available.';
                document.getElementById('api-status').className = 'api-check success';
            })
            .catch(() => {
                document.getElementById('api-status').textContent = 'API check failed';
                document.getElementById('api-status').className = 'api-check error';
            });
    </script>
</body>
</html>
"#;
