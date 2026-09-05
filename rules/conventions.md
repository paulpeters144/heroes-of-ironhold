# Coding Conventions

- All systems start with sys_*.rs for example: sys_animation.rs
- Never call `get_frame_time()`; read the frame delta time from `Context` (`ctx.dt`). `Manager` updates it once per frame.
- Never call `Assets` accessors (`get_font`, `sound`, `texture`, etc.) in `update()` or `draw()` methods. Extract all needed assets during construction (e.g. `new()` or `factory()`) and store the results on the struct.
