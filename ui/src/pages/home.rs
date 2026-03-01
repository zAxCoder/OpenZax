use leptos::*;
use crate::components::{ChatPanel, LeftSidebar, RightSidebar, CommandPalette};

#[component]
pub fn Home() -> impl IntoView {
    let (show_left_sidebar, set_show_left_sidebar) = create_signal(true);
    let (show_right_sidebar, set_show_right_sidebar) = create_signal(true);
    let (show_bottom_panel, set_show_bottom_panel) = create_signal(false);

    view! {
        <div class="home-page">
            <CommandPalette/>
            
            <div class="workspace">
                <Show when=move || show_left_sidebar.get()>
                    <LeftSidebar/>
                </Show>
                
                <div class="main-content">
                    <div class="top-bar">
                        <div class="top-bar-left">
                            <button
                                class="icon-button"
                                on:click=move |_| set_show_left_sidebar.update(|v| *v = !*v)
                                title="Toggle Left Sidebar (Ctrl+B)"
                            >
                                "☰"
                            </button>
                            <span class="app-title">"OpenZax"</span>
                        </div>
                        
                        <div class="top-bar-center">
                            <span class="status-indicator">
                                <span class="status-dot online"></span>
                                "Ready"
                            </span>
                        </div>
                        
                        <div class="top-bar-right">
                            <button
                                class="icon-button"
                                on:click=move |_| set_show_bottom_panel.update(|v| *v = !*v)
                                title="Toggle Bottom Panel (Ctrl+J)"
                            >
                                "⊞"
                            </button>
                            <button
                                class="icon-button"
                                on:click=move |_| set_show_right_sidebar.update(|v| *v = !*v)
                                title="Toggle Right Sidebar"
                            >
                                "⋮"
                            </button>
                        </div>
                    </div>
                    
                    <div class="content-area">
                        <ChatPanel/>
                    </div>
                    
                    <Show when=move || show_bottom_panel.get()>
                        <div class="bottom-panel">
                            <div class="panel-tabs">
                                <button class="tab-button active">"Terminal"</button>
                                <button class="tab-button">"Output"</button>
                                <button class="tab-button">"Debug"</button>
                            </div>
                            <div class="panel-content">
                                <div class="terminal-placeholder">
                                    "Terminal panel (coming soon)"
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>
                
                <Show when=move || show_right_sidebar.get()>
                    <RightSidebar/>
                </Show>
            </div>
        </div>
    }
}
