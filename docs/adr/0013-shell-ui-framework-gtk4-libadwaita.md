# ADR-0013: Shell UI Framework — GTK4 + libadwaita

**Date:** 2026-05-14
**Status:** Accepted
**Deciders:** Architecture team
**Resolves:** Follow-up noted in [ADR-0011](./0011-shell-implementation-language.md)

---

## Context

ADR-0011 settled Rust as the shell implementation language and GTK4/libadwaita as the primary candidate, with Iced as an alternative to evaluate. This ADR makes that evaluation final.

The shell (Minecrarch Shell) is a fullscreen Wayland application embedded in a Gamescope session. Its UI requirements are unusual:

- Gamepad-first focus traversal — every interactive element must be reachable without a mouse (ADR-0010)
- Fullscreen, appliance-style layout — no window decorations, no desktop metaphors
- Overlay surfaces — HUD overlays rendered above the game via wlr-layer-shell
- Long-lived process — the shell runs continuously for the entire system uptime; a crash returns the user to a broken state
- Complex navigation patterns — multi-level menus, settings panels, recovery flows, in-game overlays (Phase 2+)
- D-Bus integration — async signal handling from services (ADR-0012) must integrate with the UI event loop

The evaluation criterion stated by the project: **easiest to maintain and scale long-term**.

## Options Evaluated

### GTK4 + libadwaita (`gtk4-rs` + `libadwaita-rs`)

GTK4 is the GNOME toolkit with first-class Wayland support. libadwaita is the GNOME HIG widget library built on top of GTK4 that provides high-level navigation and layout patterns.

**Rust bindings:** `gtk4-rs` (official, maintained by the GNOME Foundation alongside the C library).

**Strengths:**
- Focus traversal is first-class: GTK4's focus management model (`focusable`, `focus-chain`, `can-focus`, keynav) maps directly to gamepad d-pad navigation. libadwaita's `AdwNavigationView` provides the exact stack-based navigation pattern used in console UIs.
- Wayland-native: GDK's Wayland backend is one of the most mature Wayland client implementations. GTK4 applications run in Gamescope without any configuration.
- API stability: GTK4 guarantees ABI/API stability within the 4.x series. `gtk4-rs` tracks releases with binding stability commitments. The project will not be broken by upstream changes.
- libadwaita provides Phase 2+ primitives out of the box: `AdwToastOverlay` (in-game notifications), `AdwPreferencesPage` (settings), `AdwNavigationView` (multi-level navigation), `AdwStatusPage` (recovery/error states).
- License: LGPL-2.1+ — compatible with all three license options under consideration (GPLv3, MIT, Apache-2.0).
- Async integration: `glib::MainContext` provides a `spawn_from_within` that allows bridging `zbus` (Tokio-based D-Bus) signals into the GTK main loop cleanly.
- Deployment: GTK4 and libadwaita are official Arch Linux packages. No custom repository needed. PKGBUILD dependencies are straightforward.

**Weaknesses:**
- GLib/GObject type system adds conceptual overhead for Rust developers unfamiliar with it.
- Compile times are higher than pure-Rust alternatives.
- GTK4 is a desktop toolkit; some default behaviors (window decorations, HiDPI negotiation) must be explicitly disabled for fullscreen appliance use.

### Iced

Pure Rust GUI framework using an Elm-like (Model-Update-View) architecture, rendering via wgpu.

**Strengths:**
- Zero C dependencies — clean Cargo dependency tree.
- Predictable state management (MVU architecture).

**Weaknesses:**
- Focus traversal for gamepads requires full custom implementation — Iced has no built-in gamepad navigation. This would need to be built from scratch and maintained indefinitely, a significant engineering cost for a project requirement that GTK4 satisfies out of the box.
- Wayland support is via winit, which has had compatibility issues with Gamescope's nested compositor mode. Resolving this would require upstream work outside this project's scope.
- API stability: Iced is at 0.x — API breaks between releases are common. Long-term maintenance burden is higher.
- Smaller community and fewer contributors familiar with it.

**Conclusion: eliminated** — insufficient gamepad navigation support and uncertain Wayland/Gamescope compatibility.

### Slint

Declarative GUI framework targeting embedded and appliance UIs, using a `.slint` DSL.

**Strengths:**
- The use case (embedded appliance, kiosk) aligns well with MinecrarchOS's intent.
- Built-in focus navigation for keyboard/gamepad.
- Small binary size and fast startup.

**Weaknesses:**
- License risk: Slint's open-source licensing is GPL-3.0. MinecrarchOS's license is undecided between GPLv3, MIT, and Apache-2.0. A GPL-3.0 dependency would force GPLv3 for the shell, constraining the project's licensing options before the governance decision is made.
- The `.slint` DSL is a domain-specific language contributors must learn in addition to Rust — increases onboarding friction.
- Smaller community than GTK4; fewer contributors with existing experience.
- Less proven in Gamescope/Wayland embedded compositor scenarios.

**Conclusion: eliminated** — license constraint risk and community size.

### egui (immediate mode)

Immediate mode GUI for debug/developer tooling.

**Weaknesses:** Not designed for production shell UIs with complex persistent state. Eliminated immediately.

## Decision

We will use **GTK4 + libadwaita** as the Minecrarch Shell UI framework, with the Rust bindings `gtk4-rs` and `libadwaita-rs`.

**Key implementation choices:**
- Window type: `AdwApplicationWindow` (fullscreen, no decorations)
- Navigation pattern: `AdwNavigationView` for menu stack navigation
- Overlay type: separate `wlr-layer-shell` surface managed by `services/overlay` (not a GTK window)
- Event loop integration: `glib::MainContext` + `async-channel` to bridge `zbus` async signals into GTK's main loop
- Gamepad input: raw gamepad events captured via libinput or SDL2 (outside GTK), translated into GTK focus navigation calls (`gtk::Widget::grab_focus`, `gtk::DirectionType`)
- Styling: libadwaita CSS with `AdwStyleManager` for dark/light mode (default: dark, appropriate for gaming appliance)

## Consequences

### Positive

- Gamepad focus traversal is supported by the framework from day one. No custom navigation engine needed.
- libadwaita Phase 2+ widgets (`AdwToastOverlay`, `AdwPreferencesPage`, `AdwNavigationView`) reduce shell UI development effort significantly.
- Long-term API stability — the project does not need to track breaking upstream changes as aggressively as with Iced or Slint.
- GTK4/libadwaita are well-packaged on Arch Linux. PKGBUILD dependencies are declarative and reproducible.
- LGPL license does not constrain MinecrarchOS's own license choice.

### Negative

- GLib/GObject type system (signals, properties, weak references) requires contributors to understand a paradigm that is uncommon in pure Rust codebases.
- GTK main loop + Tokio integration requires explicit bridging (`glib::MainContext::channel`, `async-channel`). Not a blocking issue, but it must be implemented correctly at project initialization.
- GTK4 is not designed for fullscreen gaming UIs out of the box: contributors must explicitly disable window decorations, set fullscreen mode via Wayland XDG-Toplevel, and suppress desktop-centric behaviors.

### Neutral

- The shell crate (`minecrarch-shell`) will depend on: `gtk4`, `libadwaita`, `zbus`, `tokio` (for D-Bus async), `async-channel` (for GTK/async bridging).
- The `services/overlay` crate is NOT a GTK application. It renders overlay surfaces via `wlr-layer-shell` using a lightweight Wayland client (likely `smithay-client-toolkit`), independent of GTK.
- Gamepad raw input is handled outside GTK via libinput or SDL2 bindings, translated into GTK focus-change calls. GTK does not need to "see" raw gamepad events.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
