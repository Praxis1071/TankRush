from pathlib import Path

p = Path("src/main.rs")
s = p.read_text()
old = """const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.25, 0.95, 0.35),
    (0.95, 0.20, 0.22),
    (0.20, 0.55, 1.0),
    (1.0, 0.82, 0.18),
];"""
new = """const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
];"""
if old not in s:
    raise SystemExit("PLAYER_COLORS block not found")
s = s.replace(old, new, 1)
old_ai = """        let color = if runtime.ai_id == Some(tank.player_id) {
            (0.38, 0.39, 0.41)
        } else {
            PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()]
        };"""
if old_ai not in s:
    raise SystemExit("tank color selection block not found")
s = s.replace(old_ai, "        let color = PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()];", 1)
s = s.replace("context.set_source_rgb(0.70, 0.06, 0.07);", "context.set_source_rgb(1.0, 0.86, 0.10);", 1)
p.write_text(s)
