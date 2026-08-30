# Camera Follow Dead-Zone (Research)

This document records the investigation into the "vertical panning locks" bug in
the `battle_test` follow camera. It documents the symptom, the root cause, the
evidence used to confirm it, and the candidate fixes.

## Symptom

In `battle_test`, the camera follows the knight (via the camera orb). Vertical
panning intermittently "locks": the camera stops panning up/down and stays frozen
at a fixed offset from the target, even though the knight is still within the
valid (unclamped) pan range. The lock does not happen on the first frame, only
after the camera has been following for a while.

## Relevant code

`src/scene/battle_test/sys_camera.rs`:

```rust
const CAMERA_SMOOTHING: f32 = 3.5;
const CAMERA_SNAP_THRESHOLD: f32 = 10.0;

fn update(&mut self, ctx: &mut Context) {
    let Some(body) = self.locate_orb() else { return; };

    let target = Vec2::new(
        self.clamp_axis(body.x, self.map_w, self.view_w),
        self.clamp_axis(body.y, self.map_h, self.view_h),
    );

    if self.snap_next {
        ctx.cam_target = target.round();
        self.snap_next = false;
        return;
    }

    let delta = target - ctx.cam_target;
    if delta.length() <= CAMERA_SNAP_THRESHOLD {
        ctx.cam_target = target.round();
    } else {
        let k = 1.0 - (-ctx.dt * CAMERA_SMOOTHING).exp();
        ctx.cam_target = ctx.cam_target.lerp(target, k).round();
    }
}
```

The camera decides between two behaviors based on the **2D** distance to the
target:

1. **Snap** (when `delta.length() <= 10`): set `cam_target = target.round()`.
2. **Lerp** (otherwise): move a fraction of the way toward the target and round
   the result to whole pixels.

## Root cause

The snap threshold and the lerp's rounding create a set of offsets where the
camera can make **zero progress forever**:

- The snap branch only fires when the 2D distance is `<= 10`.
- In the lerp branch, each axis moves by `round(delta_axis * k)` and is then
  rounded to a whole pixel. At 60fps, `k = 1 - e^(-3.5/60) ~= 0.0567`, so
  `round(8 * 0.0567) = round(0.45) = 0`. A per-axis offset of `<= 8` produces
  zero movement in that axis.

The dead zone is the set of integer offsets where the 2D length exceeds 10 (so
the snap branch is never taken) while every axis is `<= 8` (so the lerp rounds to
zero):

| delta | length   | snaps? | lerp moves?   |
|-------|----------|--------|---------------|
| (7,8) | 10.63    | no     | no (stuck)    |
| (8,7) | 10.63    | no     | no (stuck)    |
| (8,8) | 11.31    | no     | no (stuck)    |

The maximum dead-zone length is `sqrt(8^2 + 8^2) = 11.31`. Any offset with length
`> 11.31` must have at least one axis `>= 9`, and `round(9 * 0.0567) = round(0.51)
= 1`, so those offsets always move. The dead zone is therefore bounded by length
`11.31`, but the snap threshold stops at `10.0`, leaving a gap.

## Confirmation

A numerical simulation of the exact update math confirms the lock. Starting the
camera at `(0,0)` and lerping toward a static target at `(100,100)`:

```text
i=40  cam=(92, 92)  delta=(8, 8)  len=11.31  LERP
STUCK at frame 41, cam=(92, 92), delta=(8, 8), len=11.31
```

The camera converges to `(92,92)` and stops `11.3px` short of the target,
never reaching it. Enumerating all integer deltas confirms exactly three dead
offsets: `(7,8)`, `(8,7)`, `(8,8)`.

## Why it locks "not the first time"

- The first frame snaps exactly to the target via `snap_next`, so there is no
  initial lock.
- The lock only appears once the camera has been *lerping* and settles into a
  dead-zone offset. This happens when the target stops moving while diagonally
  offset from the camera — for example after a facing flip (the orb jumps
  `±100px` horizontally and the camera lerps back), or after diagonal movement.
  Once the camera rests at `(8,8)`, `(8,7)`, or `(7,8)` it freezes in both axes
  (including vertical) until the target moves far enough to break out.

## Candidate fixes

### Option A — Raise the snap threshold (minimal)

Set `CAMERA_SNAP_THRESHOLD` to `12.0`. Since the dead zone is bounded by length
`11.31`, any offset with length `<= 12` snaps, and any offset beyond it has an
axis `>= 9` that moves. This is a one-line change, but the constant is now tied
to the dead-zone size and can silently regress if `CAMERA_SMOOTHING` changes.

### Option B — Guarantee a minimum lerp step (robust)

In the lerp branch, if the rounded result equals the current position but the
target has not been reached, step one pixel toward the target. This guarantees
convergence regardless of the smoothing constant and eliminates the dead zone by
construction.

### Option C — Snap per-axis

Replace the 2D `delta.length()` check with independent per-axis checks so each
axis snaps when close and lerps otherwise. This removes the 2D coupling that
creates the gap, though the lerp rounding can still round a small step to zero
and should be combined with Option B's minimum step.

## Secondary observation (not the lock)

The map is `400px` tall while the view is `360px` tall, so `clamp_axis` caps
vertical camera travel to `[180, 220]` — only `40px` of pan range. This is
correct map-edge clamping, but if more vertical room was expected, either the
map needs to be taller or the view shorter. This is separate from the dead-zone
bug described above.
