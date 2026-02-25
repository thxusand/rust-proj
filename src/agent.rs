use crate::core::{AgentError, AgentId, Message, Position, State};
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};

/// Структура патрульного агента.
pub struct Agent {
    id: AgentId,
    state: State,
    position: Position,
}

impl Agent {
    /// Створює нового агента в базовому стані.
    pub fn new(id: AgentId, start_pos: Position) -> Self {
        Self {
            id,
            state: State::Idle,
            position: start_pos,
        }
    }

    /// Обробляє вхідну команду та змінює внутрішній стан (State Machine).
    pub fn process_command(&mut self, cmd: &Message) -> Result<(), AgentError> {
        match cmd {
            Message::MoveTo(new_pos) => {
                self.state = State::Moving;
                self.position = *new_pos;
                info!(agent_id = self.id, "Перехід у стан Moving до {:?}", new_pos);
            }
            Message::ScanSector => {
                self.state = State::Scanning;
                info!(agent_id = self.id, "Перехід у стан Scanning");
            }
            Message::StatusReport(..) => {
                warn!(agent_id = self.id, "Отримано власний репорт, ігнорую");
            }
        }
        Ok(())
    }

    /// Асинхронний життєвий цикл агента.
    pub async fn run(
        mut self,
        mut rx: broadcast::Receiver<Message>,
        tx: mpsc::Sender<Message>,
    ) -> Result<(), AgentError> {
        info!(
            agent_id = self.id,
            "Агент ініціалізовано та готовий до роботи"
        );

        // Слухаємо команди з broadcast каналу
        while let Ok(msg) = rx.recv().await {
            self.process_command(&msg)?;

            // Відправляємо статус назад координатору (graceful error handling)
            let report = Message::StatusReport(self.id, self.state, self.position);
            if tx.send(report).await.is_err() {
                return Err(AgentError::ChannelError(
                    "Втрачено зв'язок з координатором".into(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let agent = Agent::new(1, Position { x: 0.0, y: 0.0 });
        assert_eq!(agent.state, State::Idle);
    }

    #[test]
    fn test_state_transitions() {
        let mut agent = Agent::new(2, Position { x: 10.0, y: 10.0 });

        let target = Position { x: 50.0, y: 50.0 };
        assert!(agent.process_command(&Message::MoveTo(target)).is_ok());
        assert_eq!(agent.state, State::Moving);
        assert_eq!(agent.position, target);

        assert!(agent.process_command(&Message::ScanSector).is_ok());
        assert_eq!(agent.state, State::Scanning);
    }
}
