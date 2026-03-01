use leptos::*;
use web_sys::KeyboardEvent;

#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub shortcut: Option<String>,
}

#[component]
pub fn CommandPalette() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_index, set_selected_index) = create_signal(0);

    // Sample commands
    let commands = vec![
        Command {
            id: "toggle-sidebar".to_string(),
            name: "Toggle Sidebar".to_string(),
            description: "Show or hide the sidebar".to_string(),
            category: "View".to_string(),
            shortcut: Some("Ctrl+B".to_string()),
        },
        Command {
            id: "new-chat".to_string(),
            name: "New Chat".to_string(),
            description: "Start a new conversation".to_string(),
            category: "Chat".to_string(),
            shortcut: Some("Ctrl+N".to_string()),
        },
        Command {
            id: "open-settings".to_string(),
            name: "Open Settings".to_string(),
            description: "Open application settings".to_string(),
            category: "Settings".to_string(),
            shortcut: Some("Ctrl+,".to_string()),
        },
        Command {
            id: "list-models".to_string(),
            name: "List Models".to_string(),
            description: "Show available AI models".to_string(),
            category: "Models".to_string(),
            shortcut: None,
        },
    ];

    // Filter commands based on search query
    let filtered_commands = create_memo(move |_| {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            commands.clone()
        } else {
            commands
                .iter()
                .filter(|cmd| {
                    cmd.name.to_lowercase().contains(&query)
                        || cmd.description.to_lowercase().contains(&query)
                        || cmd.category.to_lowercase().contains(&query)
                })
                .cloned()
                .collect()
        }
    });

    // Global keyboard shortcut to open palette (Ctrl+Shift+P)
    let handle_global_keydown = move |ev: KeyboardEvent| {
        if ev.ctrl_key() && ev.shift_key() && ev.key() == "P" {
            ev.prevent_default();
            set_is_open.update(|open| *open = !*open);
            set_search_query.set(String::new());
            set_selected_index.set(0);
        }
    };

    // Handle keyboard navigation in palette
    let handle_palette_keydown = move |ev: KeyboardEvent| {
        let commands_count = filtered_commands.get().len();
        
        match ev.key().as_str() {
            "Escape" => {
                ev.prevent_default();
                set_is_open.set(false);
            }
            "ArrowDown" => {
                ev.prevent_default();
                set_selected_index.update(|idx| {
                    *idx = (*idx + 1).min(commands_count.saturating_sub(1));
                });
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_selected_index.update(|idx| {
                    *idx = idx.saturating_sub(1);
                });
            }
            "Enter" => {
                ev.prevent_default();
                let commands = filtered_commands.get();
                if let Some(cmd) = commands.get(selected_index.get()) {
                    tracing::info!("Executing command: {}", cmd.id);
                    // TODO: Execute command
                    set_is_open.set(false);
                }
            }
            _ => {}
        }
    };

    // Add global event listener
    create_effect(move |_| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
            handle_global_keydown(ev);
        }) as Box<dyn FnMut(_)>);
        
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        
        closure.forget();
    });

    view! {
        <Show when=move || is_open.get()>
            <div class="command-palette-overlay" on:click=move |_| set_is_open.set(false)>
                <div class="command-palette" on:click=|ev| ev.stop_propagation()>
                    <input
                        type="text"
                        class="command-search"
                        placeholder="Type a command or search..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| {
                            set_search_query.set(event_target_value(&ev));
                            set_selected_index.set(0);
                        }
                        on:keydown=handle_palette_keydown
                        autofocus
                    />
                    
                    <div class="command-list">
                        <For
                            each=move || filtered_commands.get().into_iter().enumerate()
                            key=|(idx, cmd)| (idx.clone(), cmd.id.clone())
                            children=move |(idx, cmd): (usize, Command)| {
                                let is_selected = move || idx == selected_index.get();
                                
                                view! {
                                    <div
                                        class={move || if is_selected() { "command-item selected" } else { "command-item" }}
                                        on:click=move |_| {
                                            tracing::info!("Executing command: {}", cmd.id);
                                            // TODO: Execute command
                                            set_is_open.set(false);
                                        }
                                        on:mouseenter=move |_| set_selected_index.set(idx)
                                    >
                                        <div class="command-main">
                                            <div class="command-name">{cmd.name.clone()}</div>
                                            <div class="command-description">{cmd.description.clone()}</div>
                                        </div>
                                        <div class="command-meta">
                                            <span class="command-category">{cmd.category.clone()}</span>
                                            {cmd.shortcut.clone().map(|shortcut| view! {
                                                <span class="command-shortcut">{shortcut}</span>
                                            })}
                                        </div>
                                    </div>
                                }
                            }
                        />
                        
                        {move || {
                            if filtered_commands.get().is_empty() {
                                view! {
                                    <div class="command-empty">
                                        "No commands found"
                                    </div>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }
                        }}
                    </div>
                    
                    <div class="command-footer">
                        <span>"↑↓ Navigate"</span>
                        <span>"↵ Execute"</span>
                        <span>"Esc Close"</span>
                    </div>
                </div>
            </div>
        </Show>
    }
}
