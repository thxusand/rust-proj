mod agent;
mod core;

use crate::agent::Agent;
use crate::core::{Message, Position};
use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

/// Структура для зчитування зовнішньої конфігурації.
#[derive(Debug, Deserialize)]
struct Config {
    swarm: SwarmConfig,
}

#[derive(Debug, Deserialize)]
struct SwarmConfig {
    total_agents: u32,
    sector_size: f64,
    broadcast_capacity: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Ініціалізація структурованого логування (tracing)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Не вдалося встановити tracing subscriber");

    // 2. Завантаження зовнішньої конфігурації (graceful handling)
    let settings = config::Config::builder()
        .add_source(config::File::with_name("Settings.toml"))
        .build()
        .context("Не знайдено або пошкоджено файл Settings.toml")?;

    let config: Config = settings.try_deserialize()?;
    info!("Конфігурація завантажена: {} агентів", config.swarm.total_agents);

    // 3. Ініціалізація каналів (Coordinator setup)
    // Broadcast для розсилки команд всім агентам
    let (cmd_tx, _) = broadcast::channel::<Message>(config.swarm.broadcast_capacity);
    // MPSC для збору статусів від агентів до координатора
    let (status_tx, mut status_rx) = mpsc::channel::<Message>(32);

    // 4. Розподіл агентів по секторах
    for id in 1..=config.swarm.total_agents {
        let start_pos = Position {
            x: (id as f64) * config.swarm.sector_size,
            y: 0.0,
        };

        let agent = Agent::new(id, start_pos);
        let rx = cmd_tx.subscribe();
        let tx = status_tx.clone();

        // Запуск агента в окремому асинхронному таску
        tokio::spawn(async move {
            if let Err(e) = agent.run(rx, tx).await {
                error!("Роботу агента {} перервано: {}", id, e);
            }
        });
    }

    // Чекаємо, поки агенти ініціалізуються
    sleep(Duration::from_millis(100)).await;

    // 5. Broadcast команд
    info!("Координатор: Розсилка наказу на розгортання...");
    let _ = cmd_tx.send(Message::MoveTo(Position { x: 150.0, y: 200.0 }));

    sleep(Duration::from_millis(100)).await;

    info!("Координатор: Розсилка наказу на сканування...");
    let _ = cmd_tx.send(Message::ScanSector);

    // 6. Збір статусів (з таймаутом для уникнення вічного блокування)
    info!("Координатор: Очікування звітів...");
    let mut reports_collected = 0;

    while let Ok(Some(Message::StatusReport(id, state, pos))) =
        tokio::time::timeout(Duration::from_secs(2), status_rx.recv()).await
    {
        info!("Отримано статус: Агент {} у стані {:?} на координатах ({}, {})", id, state, pos.x, pos.y);
        reports_collected += 1;
        if reports_collected == config.swarm.total_agents * 2 {
            break; // Зібрали всі відповіді на 2 команди
        }
    }

    info!("Місія завершена. Зібрано {} звітів.", reports_collected);
    Ok(())
}