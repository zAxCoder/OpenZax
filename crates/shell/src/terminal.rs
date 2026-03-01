use colored::Colorize;
use openzax_core::{
    agent::{Agent, AgentConfig},
    event::{Event, EventBus},
    storage::Storage,
    Result,
};
use std::io::{self, Write};
use uuid::Uuid;
use chrono::Utc;

pub struct TerminalShell {
    agent: Agent,
    storage: Storage,
    event_bus: EventBus,
    conversation_id: Uuid,
}

impl TerminalShell {
    pub fn new(config: AgentConfig, storage: Storage) -> Result<Self> {
        let event_bus = EventBus::default();
        let agent = Agent::new(config, event_bus.clone());
        let conversation_id = Uuid::new_v4();
        
        storage.create_conversation(conversation_id)?;

        Ok(Self {
            agent,
            storage,
            event_bus,
            conversation_id,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut event_receiver = self.event_bus.subscribe();
        
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    Event::AgentThinking { thought_text, .. } => {
                        print!(
                            "\r  {} {}",
                            "◆".truecolor(255, 180, 60),
                            thought_text.truecolor(140, 140, 170)
                        );
                        io::stdout().flush().ok();
                    }
                    Event::AgentTokenStream { token, .. } => {
                        print!("{}", token.truecolor(220, 220, 235));
                        io::stdout().flush().ok();
                    }
                    Event::AgentOutput { .. } => {
                        println!();
                        println!();
                    }
                    Event::SystemEvent { message, .. } => {
                        println!(
                            "\n  {} {}",
                            "●".truecolor(100, 180, 255),
                            message.truecolor(200, 200, 220)
                        );
                    }
                    _ => {}
                }
            }
        });

        loop {
            let id_short = &self.conversation_id.to_string()[..8];
            print!(
                " {} {} ",
                "❯".truecolor(100, 180, 255).bold(),
                id_short.truecolor(70, 70, 90),
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "exit" | "quit" | "/exit" | "/quit" => {
                    println!();
                    println!(
                        "  {} {}",
                        "●".truecolor(100, 180, 255),
                        "Goodbye!".truecolor(140, 140, 170)
                    );
                    println!();
                    break;
                }
                "help" | "/help" => {
                    self.show_help();
                    continue;
                }
                "clear" | "/clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                    continue;
                }
                _ => {}
            }

            println!();
            println!(
                "  {} {}",
                "▸".truecolor(100, 180, 255).bold(),
                input.truecolor(220, 220, 235)
            );
            println!();
            println!(
                "  {} {}",
                "▪".truecolor(255, 180, 60),
                "OpenZax".truecolor(255, 180, 60).bold()
            );

            self.storage.save_message(
                Uuid::new_v4(),
                self.conversation_id,
                "user",
                input,
            )?;

            self.event_bus.publish(Event::UserInput {
                session_id: self.conversation_id,
                content: input.to_string(),
                attachments: vec![],
                timestamp: Utc::now(),
            })?;

            print!(
                "  {} ",
                "▍".truecolor(100, 180, 255)
            );
            io::stdout().flush().ok();

            match self.agent.process_streaming(input).await {
                Ok(_) => {}
                Err(e) => {
                    println!(
                        "\n  {} {}",
                        "✗".truecolor(255, 80, 80).bold(),
                        e.to_string().truecolor(255, 120, 120)
                    );
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        let w = 68usize;
        let inner = w.saturating_sub(4);

        println!();
        println!(
            "  {}{}{}",
            "╭".truecolor(60, 60, 80),
            "─".repeat(inner).truecolor(60, 60, 80),
            "╮".truecolor(60, 60, 80),
        );

        let pad_line = |content: &str| {
            let vis = console::measure_text_width(content);
            let right_pad = inner.saturating_sub(vis + 1);
            println!(
                "  {} {}{}{}",
                "│".truecolor(60, 60, 80),
                content,
                " ".repeat(right_pad),
                "│".truecolor(60, 60, 80)
            );
        };

        pad_line(&format!("{}", " Commands".truecolor(100, 180, 255).bold()));
        pad_line(&format!(
            " {}",
            "─".repeat(inner.saturating_sub(3)).truecolor(45, 45, 60)
        ));

        let cmds: Vec<(&str, &str)> = vec![
            ("/help", "Show this help message"),
            ("/clear", "Clear the screen"),
            ("/exit", "Exit the shell"),
        ];

        for (cmd, desc) in &cmds {
            pad_line(&format!(
                " {}  {}",
                format!("{:<16}", cmd).truecolor(255, 180, 60),
                desc.truecolor(140, 140, 170),
            ));
        }

        pad_line("");
        pad_line(
            &" Or type any message to chat with the AI agent."
                .truecolor(100, 100, 130)
                .to_string(),
        );
        pad_line("");

        println!(
            "  {}{}{}",
            "╰".truecolor(60, 60, 80),
            "─".repeat(inner).truecolor(60, 60, 80),
            "╯".truecolor(60, 60, 80),
        );
        println!();
    }
}
