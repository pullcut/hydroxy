use dioxus::prelude::*;
use hydroxy_dioxus::live_cursors::{LiveCursors, RemoteCursor};

/// Where a teammate's pointer is at time `t`: a slow loop over the board, as if
/// the positions were arriving over the network.
fn path(seed: f64, t: f64) -> (f64, f64) {
    (0.5 + 0.36 * (t * 0.7 + seed).sin(), 0.5 + 0.3 * (t * 1.1 + seed * 2.0).cos())
}

#[component]
pub fn LiveCursorsDemo() -> Element {
    let mut t = use_signal(|| 0.0_f64);
    let mut mine = use_signal(|| None::<(f64, f64)>);
    use_future(move || async move {
        loop {
            let _ = document::eval("await new Promise(r => setTimeout(r, 250)); return true;").join::<bool>().await;
            t += 0.25;
        }
    });
    let now = t();
    let (jx, jy) = path(0.0, now);
    let (px, py) = path(2.1, now * 0.8);
    // Sana pauses half the time, so her cursor fades while it sits still.
    let (sx, sy) = if (now / 6.0).floor() % 2.0 == 0.0 { (0.78, 0.22) } else { path(4.2, now) };
    let cursors = vec![
        RemoteCursor::new("u-jonas", "Jonas Berg", jx, jy),
        RemoteCursor::new("u-priya", "Priya Raman", px, py),
        RemoteCursor::new("u-sana", "Sana Haddad", sx, sy),
    ];
    let readout = mine().map_or_else(|| "off the board".to_string(), |(x, y)| format!("{x:.2}, {y:.2}"));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            LiveCursors { cursors, idle_ms: 2_000, on_move: move |p| mine.set(p),
                div { style: "height: 280px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-sunken); display: grid; place-items: center; color: var(--text-3); font: var(--text-sm);",
                    "Launch retro whiteboard"
                }
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "Your pointer, as sent to others: {readout}" }
        }
    }
}
