---
name: game-screenshot
description: Take screenshots of the running game window. Use when the user asks to see the game, review visual changes, or capture the current game state.
---

# Game Screenshot

Capture screenshots of the running Heroes of Ironhold game window.

## Prerequisites

Install required tools (one-time setup):

```bash
sudo apt-get install -y wmctrl imagemagick
```

- `wmctrl` - lists windows and finds the game window by title
- `imagemagick` - provides the `import` command to capture a specific window

## Steps

1. **Launch the game** in the background:

   ```bash
   cargo run -p heroes-of-ironhold-desktop &
   GAME_PID=$!
   ```

2. **Wait for the window to appear** (8-10 seconds):

   ```bash
   sleep 8
   ```

3. **Find the window ID**:

   ```bash
   wmctrl -l | grep "Heroes of Ironhold"
   ```

   The output will show lines like:
   ```
   0x03200005  0 paul-ubuntu-OMEN-by-HP-Laptop-15-dc1xxx Heroes of Ironhold
   ```

   Copy the window ID (first column, e.g., `0x03200005`).

4. **Capture the window**:

   ```bash
   import -window <WINDOW_ID> /tmp/game_screenshot.png
   ```

5. **Clean up** - kill the game process:

   ```bash
   kill $GAME_PID 2>/dev/null
   wait $GAME_PID 2>/dev/null
   ```

6. **Read the screenshot** using the Read tool to show it to the user.

## Full Command

```bash
cargo run -p heroes-of-ironhold-desktop &
GAME_PID=$!
sleep 8
WINDOW_ID=$(wmctrl -l | grep "Heroes of Ironhold" | head -1 | awk '{print $1}')
import -window $WINDOW_ID /tmp/game_screenshot.png
echo "Screenshot saved to /tmp/game_screenshot.png"
kill $GAME_PID 2>/dev/null
wait $GAME_PID 2>/dev/null
```

## Notes

- The game must be run from the repo root (`/home/paul-ubuntu/repos/heroes-of-ironhold`)
- If `import` fails, try `scrot /tmp/game_screenshot.png` as a fallback (captures the entire screen)
- The screenshot saves to `/tmp/game_screenshot.png` which can be read with the Read tool
- If multiple game windows exist, use `head -1` to grab the first one
- The game window title is "Heroes of Ironhold"
