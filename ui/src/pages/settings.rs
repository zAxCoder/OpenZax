use leptos::*;
use leptos_router::*;

#[component]
pub fn Settings() -> impl IntoView {
    let (api_key, set_api_key) = create_signal(String::new());
    let (selected_model, set_selected_model) = create_signal("gpt-4".to_string());
    let (temperature, set_temperature) = create_signal(0.7);

    view! {
        <div class="settings-page">
            <div class="settings-header">
                <A href="/" class="back-button">"← Back"</A>
                <h1>"Settings"</h1>
            </div>
            
            <div class="settings-content">
                <section class="settings-section">
                    <h2>"API Configuration"</h2>
                    
                    <div class="setting-item">
                        <label for="api-key">"API Key"</label>
                        <input
                            id="api-key"
                            type="password"
                            class="setting-input"
                            placeholder="Enter your API key"
                            prop:value=move || api_key.get()
                            on:input=move |ev| set_api_key.set(event_target_value(&ev))
                        />
                        <p class="setting-description">
                            "Your OpenAI API key for cloud model access"
                        </p>
                    </div>
                    
                    <div class="setting-item">
                        <label for="model">"Default Model"</label>
                        <select
                            id="model"
                            class="setting-select"
                            on:change=move |ev| set_selected_model.set(event_target_value(&ev))
                        >
                            <option value="gpt-4" selected=move || selected_model.get() == "gpt-4">
                                "GPT-4"
                            </option>
                            <option value="gpt-4-turbo" selected=move || selected_model.get() == "gpt-4-turbo">
                                "GPT-4 Turbo"
                            </option>
                            <option value="gpt-3.5-turbo" selected=move || selected_model.get() == "gpt-3.5-turbo">
                                "GPT-3.5 Turbo"
                            </option>
                        </select>
                    </div>
                    
                    <div class="setting-item">
                        <label for="temperature">"Temperature: "{move || format!("{:.1}", temperature.get())}</label>
                        <input
                            id="temperature"
                            type="range"
                            min="0"
                            max="2"
                            step="0.1"
                            class="setting-slider"
                            prop:value=move || temperature.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                    set_temperature.set(val);
                                }
                            }
                        />
                        <p class="setting-description">
                            "Controls randomness in responses (0 = deterministic, 2 = very random)"
                        </p>
                    </div>
                </section>
                
                <section class="settings-section">
                    <h2>"Appearance"</h2>
                    
                    <div class="setting-item">
                        <label for="theme">"Theme"</label>
                        <select id="theme" class="setting-select">
                            <option value="midnight" selected>"Midnight (Default)"</option>
                            <option value="daylight">"Daylight"</option>
                            <option value="solarized">"Solarized Dark"</option>
                            <option value="high-contrast">"High Contrast"</option>
                        </select>
                    </div>
                    
                    <div class="setting-item">
                        <label>
                            <input type="checkbox" class="setting-checkbox"/>
                            "Enable animations"
                        </label>
                    </div>
                    
                    <div class="setting-item">
                        <label>
                            <input type="checkbox" class="setting-checkbox" checked/>
                            "Show line numbers in code blocks"
                        </label>
                    </div>
                </section>
                
                <section class="settings-section">
                    <h2>"Advanced"</h2>
                    
                    <div class="setting-item">
                        <label>
                            <input type="checkbox" class="setting-checkbox"/>
                            "Enable local model support"
                        </label>
                        <p class="setting-description">
                            "Use llama.cpp for local model inference"
                        </p>
                    </div>
                    
                    <div class="setting-item">
                        <label>
                            <input type="checkbox" class="setting-checkbox" checked/>
                            "Enable WASM skill sandboxing"
                        </label>
                        <p class="setting-description">
                            "Run skills in isolated WASM sandbox (recommended)"
                        </p>
                    </div>
                    
                    <div class="setting-item">
                        <label for="max-tokens">"Max Tokens"</label>
                        <input
                            id="max-tokens"
                            type="number"
                            class="setting-input"
                            value="2000"
                            min="100"
                            max="8000"
                        />
                    </div>
                </section>
                
                <div class="settings-actions">
                    <button class="button button-primary">"Save Settings"</button>
                    <button class="button button-secondary">"Reset to Defaults"</button>
                </div>
            </div>
        </div>
    }
}
