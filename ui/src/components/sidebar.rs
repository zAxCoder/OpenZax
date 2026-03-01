use leptos::*;

#[component]
pub fn LeftSidebar() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("explorer");

    view! {
        <div class="left-sidebar">
            <div class="sidebar-tabs">
                <button
                    class={move || if active_tab.get() == "explorer" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("explorer")
                >
                    "Explorer"
                </button>
                <button
                    class={move || if active_tab.get() == "skills" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("skills")
                >
                    "Skills"
                </button>
                <button
                    class={move || if active_tab.get() == "mcp" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("mcp")
                >
                    "MCP"
                </button>
            </div>

            <div class="sidebar-content">
                {move || match active_tab.get() {
                    "explorer" => view! { <ExplorerPanel/> }.into_view(),
                    "skills" => view! { <SkillsPanel/> }.into_view(),
                    "mcp" => view! { <McpPanel/> }.into_view(),
                    _ => view! { <div>"Unknown tab"</div> }.into_view(),
                }}
            </div>
        </div>
    }
}

#[component]
fn ExplorerPanel() -> impl IntoView {
    view! {
        <div class="explorer-panel">
            <div class="panel-header">"File Explorer"</div>
            <div class="panel-content">
                <div class="file-tree">
                    <div class="file-item folder">"📁 src"</div>
                    <div class="file-item file">"  📄 main.rs"</div>
                    <div class="file-item file">"  📄 lib.rs"</div>
                    <div class="file-item folder">"📁 docs"</div>
                    <div class="file-item file">"  📄 README.md"</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SkillsPanel() -> impl IntoView {
    view! {
        <div class="skills-panel">
            <div class="panel-header">"Installed Skills"</div>
            <div class="panel-content">
                <div class="skills-list">
                    <div class="skill-item">
                        <div class="skill-name">"hello-skill"</div>
                        <div class="skill-version">"v1.0.0"</div>
                    </div>
                    <div class="empty-state">
                        "No skills installed yet"
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn McpPanel() -> impl IntoView {
    view! {
        <div class="mcp-panel">
            <div class="panel-header">"MCP Servers"</div>
            <div class="panel-content">
                <div class="mcp-servers-list">
                    <div class="empty-state">
                        "No MCP servers connected"
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RightSidebar() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("context");

    view! {
        <div class="right-sidebar">
            <div class="sidebar-tabs">
                <button
                    class={move || if active_tab.get() == "context" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("context")
                >
                    "Context"
                </button>
                <button
                    class={move || if active_tab.get() == "activity" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("activity")
                >
                    "Activity"
                </button>
                <button
                    class={move || if active_tab.get() == "permissions" { "tab-button active" } else { "tab-button" }}
                    on:click=move |_| set_active_tab.set("permissions")
                >
                    "Permissions"
                </button>
            </div>

            <div class="sidebar-content">
                {move || match active_tab.get() {
                    "context" => view! { <ContextPanel/> }.into_view(),
                    "activity" => view! { <ActivityPanel/> }.into_view(),
                    "permissions" => view! { <PermissionsPanel/> }.into_view(),
                    _ => view! { <div>"Unknown tab"</div> }.into_view(),
                }}
            </div>
        </div>
    }
}

#[component]
fn ContextPanel() -> impl IntoView {
    view! {
        <div class="context-panel">
            <div class="panel-header">"Active Context"</div>
            <div class="panel-content">
                <div class="context-items">
                    <div class="empty-state">
                        "No active context"
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ActivityPanel() -> impl IntoView {
    view! {
        <div class="activity-panel">
            <div class="panel-header">"Agent Activity"</div>
            <div class="panel-content">
                <div class="activity-feed">
                    <div class="empty-state">
                        "No recent activity"
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PermissionsPanel() -> impl IntoView {
    view! {
        <div class="permissions-panel">
            <div class="panel-header">"Permissions"</div>
            <div class="panel-content">
                <div class="permissions-list">
                    <div class="empty-state">
                        "No active permissions"
                    </div>
                </div>
            </div>
        </div>
    }
}
