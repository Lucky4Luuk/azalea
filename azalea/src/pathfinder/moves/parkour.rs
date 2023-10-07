use azalea_client::{SprintDirection, StartSprintEvent, StartWalkEvent, WalkDirection};
use azalea_core::{direction::CardinalDirection, position::BlockPos};

use crate::{
    pathfinder::{astar, costs::*},
    JumpEvent, LookAtEvent,
};

use super::{default_is_reached, Edge, ExecuteCtx, IsReachedCtx, MoveData, PathfinderCtx};

pub fn parkour_move(edges: &mut Vec<Edge>, ctx: &PathfinderCtx, node: BlockPos) {
    parkour_forward_1_move(edges, ctx, node);
    parkour_forward_2_move(edges, ctx, node);
}

fn parkour_forward_1_move(edges: &mut Vec<Edge>, ctx: &PathfinderCtx, pos: BlockPos) {
    for dir in CardinalDirection::iter() {
        let gap_offset = BlockPos::new(dir.x(), 0, dir.z());
        let offset = BlockPos::new(dir.x() * 2, 0, dir.z() * 2);

        // make sure we actually have to jump
        if ctx.is_block_solid((pos + gap_offset).down(1)) {
            continue;
        }
        if !ctx.is_passable(pos + gap_offset) {
            continue;
        }

        let ascend: i32 = if ctx.is_standable(pos + offset.up(1)) {
            // ascend
            1
        } else if ctx.is_standable(pos + offset) {
            // forward
            0
        } else {
            continue;
        };

        // make sure we have space to jump
        if !ctx.is_block_passable((pos + gap_offset).up(2)) {
            continue;
        }

        // make sure it's not a headhitter
        if !ctx.is_block_passable(pos.up(2)) {
            continue;
        }

        let cost = JUMP_PENALTY + WALK_ONE_BLOCK_COST * 2.;

        edges.push(Edge {
            movement: astar::Movement {
                target: pos + offset.up(ascend),
                data: MoveData {
                    execute: &execute_parkour_move,
                    is_reached: &parkour_is_reached,
                },
            },
            cost,
        })
    }
}

fn parkour_forward_2_move(edges: &mut Vec<Edge>, ctx: &PathfinderCtx, pos: BlockPos) {
    for dir in CardinalDirection::iter() {
        let gap_1_offset = BlockPos::new(dir.x(), 0, dir.z());
        let gap_2_offset = BlockPos::new(dir.x() * 2, 0, dir.z() * 2);
        let offset = BlockPos::new(dir.x() * 3, 0, dir.z() * 3);

        // make sure we actually have to jump
        if ctx.is_block_solid((pos + gap_1_offset).down(1))
            || ctx.is_block_solid((pos + gap_2_offset).down(1))
        {
            continue;
        }

        if !ctx.is_standable(pos + offset) {
            continue;
        }
        if !ctx.is_passable(pos + gap_1_offset) {
            continue;
        }
        if !ctx.is_block_passable((pos + gap_1_offset).up(2)) {
            continue;
        }
        if !ctx.is_passable(pos + gap_2_offset) {
            continue;
        }
        if !ctx.is_block_passable((pos + gap_2_offset).up(2)) {
            continue;
        }
        // make sure it's not a headhitter
        if !ctx.is_block_passable(pos.up(2)) {
            continue;
        }

        let cost = JUMP_PENALTY + WALK_ONE_BLOCK_COST * 3.;

        edges.push(Edge {
            movement: astar::Movement {
                target: pos + offset,
                data: MoveData {
                    execute: &execute_parkour_move,
                    is_reached: &default_is_reached,
                },
            },
            cost,
        })
    }
}

fn execute_parkour_move(
    ExecuteCtx {
        entity,
        position,
        target,
        start,
        look_at_events,
        sprint_events,
        walk_events,
        jump_events,
        ..
    }: ExecuteCtx,
) {
    let start_center = start.center();
    let target_center = target.center();
    look_at_events.send(LookAtEvent {
        entity,
        position: target_center,
    });

    let jump_distance = i32::max((target - start).x.abs(), (target - start).z.abs());

    if jump_distance >= 4 {
        // 3 block gap
        sprint_events.send(StartSprintEvent {
            entity,
            direction: SprintDirection::Forward,
        });
    } else {
        walk_events.send(StartWalkEvent {
            entity,
            direction: WalkDirection::Forward,
        });
    }

    let x_dir = (target.x - start.x).clamp(-1, 1);
    let z_dir = (target.z - start.z).clamp(-1, 1);
    let dir = BlockPos::new(x_dir, 0, z_dir);
    let jump_at_pos = start + dir;

    let is_at_start_block = BlockPos::from(position) == start;
    let is_at_jump_block = BlockPos::from(position) == jump_at_pos;

    let required_distance_from_center = if jump_distance <= 2 {
        // 1 block gap
        0.
    } else {
        0.6
    };
    let distance_from_start = f64::max(
        (position.x - start_center.x).abs(),
        (position.z - start_center.z).abs(),
    );

    if !is_at_start_block && is_at_jump_block && distance_from_start > required_distance_from_center
    {
        jump_events.send(JumpEvent { entity });
    }
}

#[must_use]
pub fn parkour_is_reached(
    IsReachedCtx {
        position, target, ..
    }: IsReachedCtx,
) -> bool {
    // 0.094 and not 0 for lilypads
    BlockPos::from(position) == target && (position.y - target.y as f64) < 0.094
}
