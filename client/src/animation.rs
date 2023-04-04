use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    lens::TransformScaleLens, Animator, AnimatorState, EaseFunction, Tween, Tweenable,
};

const PIECE_TWEEN_TIME: f32 = 0.1;
const PIECE_TWEEN_SCALE: f32 = 1.05;

fn new_grow_tween() -> Tween<Transform> {
    Tween::new(
        EaseFunction::QuarticInOut,
        Duration::from_secs_f32(PIECE_TWEEN_TIME),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::new(PIECE_TWEEN_SCALE, PIECE_TWEEN_SCALE, 1.0),
        },
    )
}

fn new_shrink_tween() -> Tween<Transform> {
    Tween::new(
        EaseFunction::QuarticInOut,
        Duration::from_secs_f32(PIECE_TWEEN_TIME),
        TransformScaleLens {
            start: Vec3::new(PIECE_TWEEN_SCALE, PIECE_TWEEN_SCALE, 1.0),
            end: Vec3::ONE,
        },
    )
}

pub fn grow(animator: &mut Animator<Transform>) {
    let progress = 1.0 - animator.tweenable().progress();
    let mut new_tween = new_grow_tween();
    new_tween.set_progress(progress);
    animator.set_tweenable(new_tween);
    animator.state = AnimatorState::Playing;
}

pub fn shrink(animator: &mut Animator<Transform>) {
    let progress = 1.0 - animator.tweenable().progress();
    let mut new_tween = new_shrink_tween();
    new_tween.set_progress(progress);
    animator.set_tweenable(new_tween);
    animator.state = AnimatorState::Playing;
}

pub fn new_piece_animator() -> Animator<Transform> {
    let mut tween = new_shrink_tween();
    tween.set_progress(1.0);
    Animator::new(tween).with_state(AnimatorState::Paused)
}
