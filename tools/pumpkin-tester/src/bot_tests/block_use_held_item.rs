use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::{server::Server, world::BlockFlags};
use pumpkin_data::{Block, dimension::Dimension, item::Item, item_stack::ItemStack};
use pumpkin_util::{GameMode, Hand, math::position::BlockPos};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

/// A block that takes the held item (campfire food, jukebox disc) must leave the hand one
/// item shorter in survival. Before, the change was dropped and the item duplicated.
pub async fn run_block_use_held_item_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let inventory = player1.inventory();

    bot1.send_command("gamemode survival").await?;
    // The server's position: earlier checks can push the player away from the bot's own.
    let pos = player1.position();
    let (x, y, z) = (
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );
    let campfire = BlockPos::new(x + 2, y, z);
    let jukebox = BlockPos::new(x - 2, y, z);
    world.set_block_state(
        &campfire,
        Block::CAMPFIRE.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    world.set_block_state(
        &jukebox,
        Block::JUKEBOX.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );

    let mut checks = [
        (campfire, ItemStack::new(5, &Item::COD), 4),
        (jukebox, ItemStack::new(1, &Item::MUSIC_DISC_CAT), 0),
    ];
    for (sequence, (pos, stack, expected)) in (10..).zip(&mut checks) {
        for _ in 0..MAX_POLL_TICKS {
            tokio::time::sleep(Duration::from_millis(5)).await;
            server.tick();
            if player1.gamemode.load() == GameMode::Survival {
                break;
            }
        }
        inventory.set_stack_in_hand(Hand::Right, stack.clone());
        bot1.use_block(*pos, sequence).await?;
        let mut count = stack.item_count;
        for _ in 0..MAX_POLL_TICKS {
            tokio::time::sleep(Duration::from_millis(5)).await;
            server.tick();
            count = inventory.held_item().item_count;
            if count == *expected {
                break;
            }
        }
        if count != *expected {
            return Err(format!(
                "Expected {} x{expected} in hand after using it on {pos:?}, got x{count}",
                stack.item.registry_key
            ));
        }
    }

    bot1.send_command("gamemode creative").await?;
    summary_lines.push(format!(
        "[PASS] bot:block_use_takes_held_item ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
