use leptos::*;
use crate::api::{ChatMessage, send_message};

#[component]
pub fn ChatPanel() -> impl IntoView {
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (input_value, set_input_value) = create_signal(String::new());
    let (is_sending, set_is_sending) = create_signal(false);

    let send_message_action = create_action(|content: &String| {
        let content = content.clone();
        async move {
            send_message(content).await
        }
    });

    let on_submit = move |ev: web_sys::Event| {
        ev.prevent_default();
        
        let content = input_value.get();
        if content.trim().is_empty() {
            return;
        }

        // Add user message
        let user_message = ChatMessage {
            role: "user".to_string(),
            content: content.clone(),
            timestamp: js_sys::Date::now() as i64 / 1000,
        };
        
        set_messages.update(|msgs| msgs.push(user_message));
        set_input_value.set(String::new());
        set_is_sending.set(true);

        // Send to backend
        send_message_action.dispatch(content);
    };

    // Handle response
    create_effect(move |_| {
        if let Some(Ok(response)) = send_message_action.value().get() {
            set_messages.update(|msgs| msgs.push(response.message));
            set_is_sending.set(false);
        }
    });

    view! {
        <div class="chat-panel">
            <div class="chat-messages">
                <For
                    each=move || messages.get()
                    key=|msg| msg.timestamp
                    children=move |msg: ChatMessage| {
                        view! {
                            <div class={format!("message message-{}", msg.role)}>
                                <div class="message-role">{msg.role.clone()}</div>
                                <div class="message-content">{msg.content.clone()}</div>
                            </div>
                        }
                    }
                />
                
                {move || is_sending.get().then(|| view! {
                    <div class="message message-assistant">
                        <div class="message-role">"assistant"</div>
                        <div class="message-content typing-indicator">
                            <span></span>
                            <span></span>
                            <span></span>
                        </div>
                    </div>
                })}
            </div>

            <form class="chat-input-form" on:submit=on_submit>
                <textarea
                    class="chat-input"
                    placeholder="Type your message... (Ctrl+Enter to send)"
                    prop:value=move || input_value.get()
                    on:input=move |ev| {
                        set_input_value.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" && ev.ctrl_key() {
                            ev.prevent_default();
                            on_submit(ev.unchecked_into());
                        }
                    }
                />
                <button
                    type="submit"
                    class="send-button"
                    disabled=move || is_sending.get()
                >
                    {move || if is_sending.get() { "Sending..." } else { "Send" }}
                </button>
            </form>
        </div>
    }
}
