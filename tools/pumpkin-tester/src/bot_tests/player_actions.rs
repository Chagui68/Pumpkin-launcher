use std::{
    sync::{Arc, PoisonError, atomic::Ordering},
    time::{Duration, Instant},
};

use pumpkin_core::{entity::player::Player, server::Server, world::World};
use pumpkin_data::{dimension::Dimension, item::Item, item_stack::ItemStack};
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_protocol::java::server::play::Action;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_player_actions_and_disconnect_tests(
    server: &Arc<Server>,
    mut bot1: HeadlessBot,
    bot2: HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let action_start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let entity_id = player1.entity_id();

    bot1.set_held_item(4).await?;
    bot1.send_player_command(entity_id, Action::StartSprinting)
        .await?;

    let mut slot_updated = false;
    let mut sprinting_updated = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if player1.inventory.get_selected_slot() == 4 {
            slot_updated = true;
        }
        if player1
            .living_entity
            .entity
            .sprinting
            .load(Ordering::Relaxed)
        {
            sprinting_updated = true;
        }
        if slot_updated && sprinting_updated {
            break;
        }
    }

    if !slot_updated {
        return Err(format!(
            "Expected TestBot1 selected hotbar slot 4 after SSetHeldItem, got {}",
            player1.inventory.get_selected_slot()
        ));
    }
    if !sprinting_updated {
        return Err("Expected TestBot1 sprinting=true after SPlayerCommand::StartSprinting".into());
    }

    bot1.send_player_command(entity_id, Action::StopSprinting)
        .await?;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if !player1
            .living_entity
            .entity
            .sprinting
            .load(Ordering::Relaxed)
        {
            break;
        }
    }

    summary_lines.push(format!(
        "[PASS] bot:held_slot_and_sprinting_actions ({}ms)",
        action_start.elapsed().as_millis()
    ));

    check_disconnect_drops(server, &world, &player1, bot1, summary_lines).await?;

    let disconnect_start = Instant::now();
    drop(bot2);
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if server.get_player_count() == 0 {
            break;
        }
    }
    let remaining_players = server.get_player_count();
    if remaining_players != 0 {
        return Err(format!(
            "Expected 0 connected players after bot disconnect, found {remaining_players}"
        ));
    }
    summary_lines.push(format!(
        "[PASS] bot:disconnect_cleanup ({}ms)",
        disconnect_start.elapsed().as_millis()
    ));

    Ok(())
}

async fn check_disconnect_drops(
    server: &Arc<Server>,
    world: &Arc<World>,
    player1: &Arc<Player>,
    bot1: HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    // Vanilla `Player.remove` drops the cursor stack and the 2x2 crafting grid where the
    // player logged out. bot2 stays to keep the area loaded while the drops are checked.
    let disconnect_start = Instant::now();
    fill_cursor_and_crafting_grid(player1);
    drop(bot1);
    let dropped = |item: &Item, count: u8| {
        world.entities.load().iter().any(|entity| {
            entity.get_item_entity().is_some_and(|item_entity| {
                let stack = item_entity
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                stack.item == item && stack.item_count == count
            })
        })
    };
    let mut drops_found = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if server.get_player_count() == 1
            && dropped(&Item::DIAMOND, 4)
            && dropped(&Item::OAK_LOG, 16)
        {
            drops_found = true;
            break;
        }
    }
    if !drops_found {
        return Err(
            "Expected the cursor stack and 2x2 crafting grid of a disconnecting player to be dropped"
                .into(),
        );
    }
    summary_lines.push(format!(
        "[PASS] bot:disconnect_drops_cursor_and_crafting_grid ({}ms)",
        disconnect_start.elapsed().as_millis()
    ));
    Ok(())
}

fn fill_cursor_and_crafting_grid(player: &Player) {
    let handler = player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    *handler
        .get_behaviour()
        .cursor_stack
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = ItemStack::new(4, &Item::DIAMOND);
    handler.get_behaviour().slots[1].set_stack(ItemStack::new(16, &Item::OAK_LOG));
}
