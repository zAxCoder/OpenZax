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
        println!("OpenZax Terminal Shell v0.1.0");
        println!("Type 'exit' to quit, 'help' for commands\n");

        let mut event_receiver = self.event_bus.subscribe();
        
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    Event::AgentThinking { thought_text, .. } => {
                        print!("\r[Thinking] {}", thought_text);
                        io::stdout().flush().ok();
                    }
                    Event::AgentTokenStream { token, .. } => {
                        print!("{}", token);
                        io::stdout().flush().ok();
                    }
                    Event::AgentOutput { .. } => {
                        println!();
                    }
                    Event::SystemEvent { message, .. } => {
                        println!("\n[System] {}", message);
                    }
                    _ => {}
                }
            }
        });

        loop {
            print!("\n> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "exit" | "quit" => {
                    println!("Goodbye!");
                    break;
                }
                "help" => {
                    self.show_help();
                    continue;
                }
                "clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                    continue;
                }
                _ => {}
            }

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

            match self.agent.process_streaming(input).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("\nError: {}", e);
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\nAvailable commands:");
        println!("  help  - Show this help message");
        println!("  clear - Clear the screen");
        println!("  exit  - Exit the shell");
        println!("\nOr type any message to chat with the AI agent.");
    }
}
