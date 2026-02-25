// СЛОВНИК
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Унікальний ідентифікатор агента в рої.
pub type AgentId = u32;

/// Двовимірна позиція агента у просторі.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Вектор швидкості агента.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

/// Стани, в яких може перебувати агент.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum State {
    Idle,
    Moving,
    Scanning,
}

/// Команди та повідомлення для спілкування між координатором та агентами.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Наказ рухатись у задану точку.
    MoveTo(Position),
    /// Наказ розпочати сканування поточного сектора.
    ScanSector,
    /// Звіт агента про свій поточний стан.
    StatusReport(AgentId, State, Position),
}

/// Глобальний тип помилок для модулів агента.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Агент {0} не зміг обробити команду через внутрішній збій")]
    ProcessingError(AgentId),
    #[error("Критична помилка каналу зв'язку: {0}")]
    ChannelError(String),
}
