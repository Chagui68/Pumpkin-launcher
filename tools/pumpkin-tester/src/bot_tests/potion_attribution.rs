use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use pumpkin_core::{entity::EntityBase, server::Server};
use pumpkin_data::{
    data_component::DataComponent, data_component_impl::PotionContentsImpl, dimension::Dimension,
    entity::EntityType, item::Item, item_stack::ItemStack, potion::Potion,
};
use pumpkin_protocol::{codec::var_int::VarInt, java::server::play::SUseItem};
use pumpkin_util::Hand;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

/// Vanilla `HealOrHarmMobEffect` hurts with `indirectMagic(potion, thrower)`, so a splash
/// potion's harm is attributed to the player who threw it.
pub async fn run_potion_attribution_test(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };

    let pos = player1.position();
    bot1.send_command(&format!(
        "summon minecraft:pig {} {} {}",
        pos.x + 1.0,
        pos.y,
        pos.z
    ))
    .await?;
    let mut pig = None;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        pig = world
            .entities
            .load()
            .iter()
            .find(|entity| entity.get_entity().entity_type.id == EntityType::PIG.id)
            .cloned();
        if pig.is_some() {
            break;
        }
    }
    let pig = pig.ok_or("Expected the summoned pig to exist")?;
    let pig_living = pig
        .get_living_entity()
        .ok_or("Expected the pig to be a living entity")?;

    let harming = ItemStack::new_with_component(
        1,
        &Item::SPLASH_POTION,
        vec![(
            DataComponent::PotionContents,
            Some(Box::new(PotionContentsImpl {
                potion_id: Some(i32::from(Potion::HARMING.id)),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            }) as Box<_>),
        )],
    );
    player1.inventory().set_stack_in_hand(Hand::Right, harming);
    // Thrown straight down, so it splashes next to the pig.
    let yaw = player1.get_entity().yaw.load();
    player1.get_entity().set_rotation(yaw, 90.0);
    bot1.send_packet(&SUseItem {
        hand: VarInt(0),
        sequence: VarInt(30),
        yaw,
        pitch: 90.0,
    })
    .await?;

    let mut attacker = 0;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        attacker = pig_living.last_hurt_by_player_id.load(Ordering::Relaxed);
        if attacker != 0 {
            break;
        }
    }
    pig.get_entity().remove();
    if attacker != player1.entity_id() {
        return Err(format!(
            "Expected the splash potion harm to be attributed to TestBot1 ({}), got {attacker}",
            player1.entity_id()
        ));
    }

    summary_lines.push(format!(
        "[PASS] bot:splash_potion_harm_attributed_to_thrower ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
