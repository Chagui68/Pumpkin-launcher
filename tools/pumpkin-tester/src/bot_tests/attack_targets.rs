use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::server::Server;
use pumpkin_data::{dimension::Dimension, entity::EntityType};
use pumpkin_protocol::{codec::var_int::VarInt, java::server::play::SAttack};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

/// Vanilla `handleAttack` ignores an unknown target and finds ender dragon parts through
/// `ServerLevel.getEntityOrPart`. The client only ever hits the parts, never the dragon.
pub async fn run_attack_target_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };

    let pos = bot1.position;
    bot1.send_command(&format!(
        "summon minecraft:ender_dragon {} {} {}",
        pos.x,
        pos.y + 4.0,
        pos.z
    ))
    .await?;
    let mut dragon = None;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        dragon = world
            .entities
            .load()
            .iter()
            .find(|entity| entity.get_entity().entity_type.id == EntityType::ENDER_DRAGON.id)
            .cloned();
        if dragon.is_some() {
            break;
        }
    }
    let dragon = dragon.ok_or("Expected the summoned ender dragon to exist")?;
    let health = || dragon.get_living_entity().map_or(0.0, |l| l.health.load());
    let health_before = health();

    // The body is the third part, after the head and the neck.
    let body_id = dragon.get_entity().entity_id + 3;
    bot1.send_packet(&SAttack {
        entity_id: VarInt(i32::MAX),
    })
    .await?;
    bot1.send_packet(&SAttack {
        entity_id: VarInt(body_id),
    })
    .await?;

    let mut damaged = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if health() < health_before {
            damaged = true;
            break;
        }
    }
    dragon.get_entity().remove();
    if player1.client.closed() {
        return Err("Attacking an unknown entity or a dragon part disconnected TestBot1".into());
    }
    if !damaged {
        return Err(format!(
            "Expected hitting dragon part {body_id} to damage the dragon (health {health_before})"
        ));
    }

    summary_lines.push(format!(
        "[PASS] bot:attack_unknown_entity_and_dragon_part ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
