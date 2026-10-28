use dioxus::prelude::*;
use oxide_dioxus::image_cropper::{Crop, ImageCropper};

#[component]
pub fn ImageCropperDemo() -> Element {
    let mut crop = use_signal(|| None::<Crop>);
    let summary = crop().map_or_else(String::new, |c| {
        format!("Export {} × {} px from ({}, {}), rotated {}°", c.width, c.height, c.x, c.y, c.rotation)
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 640px;",
            // Drag the box or its handles, pick a ratio, zoom and rotate.
            // Focus the box for arrow keys (Shift: bigger steps, Alt: resize).
            ImageCropper {
                src: PHOTO.to_string(),
                alt: "Sunset over mountains and a lake, pines in the foreground",
                label: "Crop cover photo",
                aspect: 16.0 / 9.0,
                on_change: move |c| crop.set(Some(c)),
            }
            span { style: "color: var(--text-2);", "{summary}" }
        }
    }
}

/// A 1200 × 800 landscape, inline so the demo works offline.
const PHOTO: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1200' height='800' viewBox='0 0 1200 800'%3E%3Cdefs%3E%3ClinearGradient id='s' x1='0' y1='0' x2='0' y2='1'%3E%3Cstop offset='0' stop-color='%231e3a5f'/%3E%3Cstop offset='.55' stop-color='%23f08a5d'/%3E%3Cstop offset='1' stop-color='%23fcd29f'/%3E%3C/linearGradient%3E%3ClinearGradient id='w' x1='0' y1='0' x2='0' y2='1'%3E%3Cstop offset='0' stop-color='%23f3a674'/%3E%3Cstop offset='1' stop-color='%2322405e'/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect width='1200' height='520' fill='url%28%23s%29'/%3E%3Ccircle cx='820' cy='400' r='70' fill='%23ffe6b0'/%3E%3Cpath d='M0 470 L180 300 L300 400 L460 220 L640 430 L760 330 L900 450 L1040 280 L1200 420 V520 H0Z' fill='%235b4a6b'/%3E%3Cpath d='M0 500 L140 400 L260 470 L420 360 L560 480 L700 410 L860 500 L1000 390 L1200 490 V520 H0Z' fill='%233b3352'/%3E%3Crect y='520' width='1200' height='280' fill='url%28%23w%29'/%3E%3Cellipse cx='820' cy='560' rx='90' ry='10' fill='%23ffe6b0' opacity='.6'/%3E%3Cpath d='M0 520 H1200' stroke='%23fcd29f' stroke-width='2' opacity='.5'/%3E%3Cg fill='%231c2333'%3E%3Cpath d='M60 640 L100 480 L140 640Z'/%3E%3Cpath d='M120 660 L170 450 L220 660Z'/%3E%3Cpath d='M200 650 L240 500 L280 650Z'/%3E%3Cpath d='M980 660 L1030 470 L1080 660Z'/%3E%3Cpath d='M1060 650 L1100 510 L1140 650Z'/%3E%3C/g%3E%3Cpath d='M0 700 Q300 660 600 700 T1200 690 V800 H0Z' fill='%231c2333'/%3E%3C/svg%3E";
