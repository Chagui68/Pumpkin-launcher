use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{HorizontalFacing, RailShape, RailShapeStraight},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::world::World;

use super::super::block_receives_redstone_power;
use super::{HorizontalFacingRailExt, Rail, RailElevation, RailProperties, StraightRailShapeExt};

/// `PoweredRailBlock.updateState`, shared by powered and activator rails.
pub(super) fn update_powered_rail_state(world: &Arc<World>, block: &Block, pos: &BlockPos) {
    let mut rail_props = RailProperties::new(world.get_block_state_id(pos), block);
    let should_power = block_receives_redstone_power(world, pos)
        || find_powered_rail_signal(world, block, pos, &rail_props, true, 0)
        || find_powered_rail_signal(world, block, pos, &rail_props, false, 0);

    if should_power != rail_props.is_powered() {
        rail_props.set_powered(should_power);
        world.set_block_state(pos, rail_props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        world.update_neighbors_at(&pos.down(), block, None);
        if rail_props.shape().is_ascending() {
            world.update_neighbors_at(&pos.up(), block, None);
        }
    }
}

/// `PoweredRailBlock.findPoweredRailSignal`
fn find_powered_rail_signal(
    world: &World,
    block: &Block,
    pos: &BlockPos,
    rail_props: &RailProperties,
    forward: bool,
    search_depth: u8,
) -> bool {
    if search_depth >= 8 {
        return false;
    }

    let BlockPos(mut next) = *pos;
    let mut check_below = true;
    let shape = match rail_props.shape() {
        RailShape::NorthSouth => {
            next.z += if forward { 1 } else { -1 };
            RailShape::NorthSouth
        }
        RailShape::EastWest => {
            next.x += if forward { -1 } else { 1 };
            RailShape::EastWest
        }
        RailShape::AscendingEast => {
            if forward {
                next.x -= 1;
            } else {
                next.x += 1;
                next.y += 1;
                check_below = false;
            }
            RailShape::EastWest
        }
        RailShape::AscendingWest => {
            if forward {
                next.x -= 1;
                next.y += 1;
                check_below = false;
            } else {
                next.x += 1;
            }
            RailShape::EastWest
        }
        RailShape::AscendingNorth => {
            if forward {
                next.z += 1;
            } else {
                next.z -= 1;
                next.y += 1;
                check_below = false;
            }
            RailShape::NorthSouth
        }
        RailShape::AscendingSouth => {
            if forward {
                next.z += 1;
                next.y += 1;
                check_below = false;
            } else {
                next.z -= 1;
            }
            RailShape::NorthSouth
        }
        // Straight rails never take a curved shape.
        _ => return false,
    };

    let next = BlockPos(next);
    is_same_rail_with_power(world, block, &next, forward, search_depth, shape)
        || (check_below
            && is_same_rail_with_power(world, block, &next.down(), forward, search_depth, shape))
}

/// `PoweredRailBlock.isSameRailWithPower`
fn is_same_rail_with_power(
    world: &World,
    block: &Block,
    pos: &BlockPos,
    forward: bool,
    search_depth: u8,
    dir: RailShape,
) -> bool {
    let (found, state_id) = world.get_block_and_state_id(pos);
    if found != block {
        return false;
    }

    let rail_props = RailProperties::new(state_id, block);
    let crosses = match dir {
        RailShape::EastWest => matches!(
            rail_props.shape(),
            RailShape::NorthSouth | RailShape::AscendingNorth | RailShape::AscendingSouth
        ),
        RailShape::NorthSouth => matches!(
            rail_props.shape(),
            RailShape::EastWest | RailShape::AscendingEast | RailShape::AscendingWest
        ),
        _ => false,
    };
    if crosses || !rail_props.is_powered() {
        return false;
    }

    block_receives_redstone_power(world, pos)
        || find_powered_rail_signal(world, block, pos, &rail_props, forward, search_depth + 1)
}

/// `BaseRailBlock.affectNeighborsAfterRemoval` for the straight (powered and activator) rails.
pub(super) fn affect_neighbors_after_powered_rail_removal(
    world: &Arc<World>,
    block: &Block,
    old_state_id: BlockStateId,
    pos: &BlockPos,
    moved: bool,
) {
    if moved {
        return;
    }
    if RailProperties::new(old_state_id, block)
        .shape()
        .is_ascending()
    {
        world.update_neighbors_at(&pos.up(), block, None);
    }
    world.update_neighbors_at(pos, block, None);
    world.update_neighbors_at(&pos.down(), block, None);
}

pub(super) fn rail_placement_is_valid(world: &World, block: &Block, pos: &BlockPos) -> bool {
    if !can_place_rail_at(world, pos) {
        return false;
    }

    let state_id = world.get_block_state_id(pos);
    let rail_props = RailProperties::new(state_id, block);
    let rail_leaning_direction = match rail_props.shape() {
        RailShape::AscendingNorth => Some(HorizontalFacing::North),
        RailShape::AscendingSouth => Some(HorizontalFacing::South),
        RailShape::AscendingEast => Some(HorizontalFacing::East),
        RailShape::AscendingWest => Some(HorizontalFacing::West),
        _ => None,
    };

    if let Some(direction) = rail_leaning_direction
        && !can_place_rail_at(world, &pos.offset(direction.to_offset()).up())
    {
        return false;
    }

    true
}

pub(super) fn can_place_rail_at(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let state = world.get_block_state(&pos.down());
    state.is_side_solid(BlockDirection::Up)
}

pub(super) fn compute_placed_rail_shape(
    world: &World,
    block_pos: &BlockPos,
    player_facing: HorizontalFacing,
) -> RailShapeStraight {
    // Use the same sophisticated logic as normal rails, but adapted for straight rails
    // Check each direction for rail connections, similar to normal rail placement

    // Check East first
    if let Some(east_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::East) {
        // Check for opposite connection (West) to form a straight line
        if let Some(west_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West) {
            // We have connections in both East and West
            if east_rail.elevation == RailElevation::Up {
                return RailShapeStraight::AscendingEast;
            } else if west_rail.elevation == RailElevation::Up {
                return RailShapeStraight::AscendingWest;
            }
            return RailShapeStraight::EastWest;
        }
        // Only East connection
        if east_rail.elevation == RailElevation::Up {
            return RailShapeStraight::AscendingEast;
        }
        return RailShapeStraight::EastWest;
    }

    // Check South
    if let Some(south_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::South) {
        // Check for opposite connection (North) to form a straight line
        if let Some(north_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North)
        {
            // We have connections in both South and North
            if south_rail.elevation == RailElevation::Up {
                return RailShapeStraight::AscendingSouth;
            } else if north_rail.elevation == RailElevation::Up {
                return RailShapeStraight::AscendingNorth;
            }
            return RailShapeStraight::NorthSouth;
        }
        // Only South connection
        if south_rail.elevation == RailElevation::Up {
            return RailShapeStraight::AscendingSouth;
        }
        return RailShapeStraight::NorthSouth;
    }

    // Check West
    if let Some(west_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West) {
        if west_rail.elevation == RailElevation::Up {
            return RailShapeStraight::AscendingWest;
        }
        return RailShapeStraight::EastWest;
    }

    // Check North
    if let Some(north_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North) {
        if north_rail.elevation == RailElevation::Up {
            return RailShapeStraight::AscendingNorth;
        }
        return RailShapeStraight::NorthSouth;
    }

    // No connections found, use player facing direction
    player_facing.to_rail_shape_flat()
}

pub(super) fn update_flanking_rails_shape(
    world: &Arc<World>,
    block: &Block,
    state_id: BlockStateId,
    block_pos: &BlockPos,
) {
    for direction in RailProperties::new(state_id, block).directions() {
        let Some(mut flanking_rail) =
            Rail::find_with_elevation(world, block_pos.offset(direction.to_offset()))
        else {
            // Skip non-rail blocks
            continue;
        };

        let new_shape =
            compute_flanking_rail_new_shape(world, &flanking_rail, direction.opposite());

        if new_shape != flanking_rail.properties.shape() {
            flanking_rail.properties.set_shape(new_shape);
            world.set_block_state(
                &flanking_rail.position,
                flanking_rail.properties.to_state_id(flanking_rail.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}

fn compute_flanking_rail_new_shape(
    world: &World,
    rail: &Rail,
    flanking_from: HorizontalFacing,
) -> RailShape {
    let mut connected_towards = Vec::with_capacity(2);
    let mut is_already_connected_to_elevated_rail = false;

    for neighbor_direction in rail.properties.directions() {
        if neighbor_direction == flanking_from {
            // Rails pointing to where the player placed are not connected
            continue;
        }

        let Some(maybe_connected_rail) =
            Rail::find_with_elevation(world, rail.position.offset(neighbor_direction.to_offset()))
        else {
            // Rails pointing to non-rail blocks are not connected
            continue;
        };

        if maybe_connected_rail
            .properties
            .directions()
            .into_iter()
            .any(|d| d == neighbor_direction.opposite())
        {
            // Rails pointing to other rails that are pointing back are connected
            connected_towards.push(neighbor_direction);
            is_already_connected_to_elevated_rail =
                maybe_connected_rail.elevation == RailElevation::Up;
        }
    }

    let new_neighbor_directions = match connected_towards.len() {
        2 => {
            // Do not update rails that are locked (aka fully connected)
            return rail.properties.shape();
        }
        1 => [connected_towards[0], flanking_from],
        0 => [flanking_from, flanking_from.opposite()],
        _ => {
            tracing::error!(
                "Rails only have two sides, but got {}",
                connected_towards.len()
            );
            return rail.properties.shape();
        }
    };

    // Handle rails that want to be straight
    if new_neighbor_directions
        .iter()
        .all(|d| *d == flanking_from || *d == flanking_from.opposite())
    {
        if rail.elevation == RailElevation::Down {
            if is_already_connected_to_elevated_rail {
                // Prioritize the South/West ascending
                return match flanking_from {
                    HorizontalFacing::South | HorizontalFacing::North => RailShape::AscendingSouth,
                    HorizontalFacing::West | HorizontalFacing::East => RailShape::AscendingWest,
                };
            }

            return flanking_from.to_rail_shape_ascending_towards().as_shape();
        } else if is_already_connected_to_elevated_rail {
            return connected_towards[0]
                .to_rail_shape_ascending_towards()
                .as_shape();
        }

        // Reset the shape to flat even if the rail already had good directions
        return rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1]);
    }

    // Handle straight rails that want to curve
    if !rail.properties.can_curve() {
        return if new_neighbor_directions[0] == HorizontalFacing::North
            || new_neighbor_directions[0] == HorizontalFacing::South
        {
            if rail.elevation == RailElevation::Down {
                // The rail is down so it should be ascending
                flanking_from.to_rail_shape_ascending_towards().as_shape()
            } else {
                rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1])
            }
        } else {
            rail.properties.shape()
        };
    }

    rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1])
}
