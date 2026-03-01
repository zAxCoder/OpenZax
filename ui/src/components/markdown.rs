use leptos::*;

#[component]
pub fn MarkdownRenderer(
    #[prop(into)] content: String,
) -> impl IntoView {
    // TODO: Implement actual markdown parsing and rendering
    // For now, simple text rendering with basic formatting
    
    let formatted_content = format_markdown_simple(&content);
    
    view! {
        <div class="markdown-content" inner_html=formatted_content />
    }
}

fn format_markdown_simple(content: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    
    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
                code_lang.clear();
            } else {
                code_lang = line.trim_start_matches("```").trim().to_string();
                html.push_str(&format!(
                    "<pre class=\"code-block\"><code class=\"language-{}\">",
                    code_lang
                ));
                in_code_block = true;
            }
            continue;
        }
        
        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
        } else {
            // Simple inline formatting
            let mut formatted = html_escape(line);
            
            // Bold: **text**
            formatted = formatted.replace("**", "<strong>");
            
            // Italic: *text*
            formatted = formatted.replace("*", "<em>");
            
            // Inline code: `code`
            formatted = formatted.replace("`", "<code>");
            
            // Headers
            if line.starts_with("# ") {
                html.push_str(&format!("<h1>{}</h1>", &formatted[2..]));
            } else if line.starts_with("## ") {
                html.push_str(&format!("<h2>{}</h2>", &formatted[3..]));
            } else if line.starts_with("### ") {
                html.push_str(&format!("<h3>{}</h3>", &formatted[4..]));
            } else if line.is_empty() {
                html.push_str("<br/>");
            } else {
                html.push_str(&format!("<p>{}</p>", formatted));
            }
        }
    }
    
    if in_code_block {
        html.push_str("</code></pre>");
    }
    
    html
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[component]
pub fn CodeBlock(
    #[prop(into)] code: String,
    #[prop(into, optional)] language: Option<String>,
) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);
    
    let copy_to_clipboard = move |_| {
        // TODO: Implement clipboard copy
        set_copied.set(true);
        
        set_timeout(
            move || set_copied.set(false),
            std::time::Duration::from_secs(2),
        );
    };
    
    view! {
        <div class="code-block-container">
            <div class="code-block-header">
                {language.clone().map(|lang| view! {
                    <span class="code-language">{lang}</span>
                })}
                <button
                    class="copy-button"
                    on:click=copy_to_clipboard
                >
                    {move || if copied.get() { "✓ Copied" } else { "Copy" }}
                </button>
            </div>
            <pre class="code-block">
                <code class={format!("language-{}", language.unwrap_or_default())}>
                    {code}
                </code>
            </pre>
        </div>
    }
}
