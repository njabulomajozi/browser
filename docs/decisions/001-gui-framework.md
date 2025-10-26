# Decision: GUI Framework (egui vs iced)

**Date**: 2025-10-25
**Status**: ✅ Decided - Use **egui**
**Deciders**: Browser MVP Team

---

## Context

Need to choose a GUI framework for the browser chrome (tabs, URL bar, toolbar). Must be cross-platform (Linux, macOS, Windows), integrate with wgpu for rendering, and allow embedding Servo's rendered output.

---

## Options Considered

### 1. egui (Immediate Mode)

**Pros:**
- ✅ Simple, immediate-mode API - easy to learn
- ✅ Mature and widely used (100+ projects)
- ✅ Built-in widgets (buttons, text input, panels)
- ✅ Excellent wgpu integration via eframe
- ✅ Hot reload friendly - UI rebuilds every frame
- ✅ Great documentation and examples
- ✅ Small binary size (~500KB overhead)
- ✅ Active development and community

**Cons:**
- ❌ Redraws every frame (higher CPU usage at idle)
- ❌ Less "native" look and feel
- ❌ Limited animation support

**Code Example:**

```rust
impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("url_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.url);
                if ui.button("Go").clicked() {
                    self.navigate();
                }
            });
        });
    }
}
```

### 2. iced (Declarative, Elm-inspired)

**Pros:**
- ✅ Declarative UI - more like React/Flutter
- ✅ Efficient rendering - only redraws on state change
- ✅ Better animation support
- ✅ Compile-time UI checks via types
- ✅ Lower idle CPU usage

**Cons:**
- ❌ Steeper learning curve (Elm architecture)
- ❌ Less mature than egui
- ❌ Smaller community
- ❌ More boilerplate code
- ❌ Harder to prototype quickly

**Code Example:**

```rust
impl Application for BrowserApp {
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UrlChanged(url) => {
                self.url = url;
                Command::none()
            }
            Message::Navigate => {
                self.navigate();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        row![
            text_input("URL", &self.url).on_input(Message::UrlChanged),
            button("Go").on_press(Message::Navigate),
        ].into()
    }
}
```

---

## Decision

**Use egui** for the MVP.

---

## Rationale

### Why egui?

1. **Speed of Development**: Immediate mode is faster for prototyping. Can iterate quickly during MVP phase.

2. **Simplicity**: Easier for contributors to understand and modify. Lower barrier to entry.

3. **Integration**: Excellent eframe integration with wgpu. Well-tested with custom rendering (Servo output).

4. **Community**: Larger user base, more examples, better documentation.

5. **MVP Focus**: For MVP, development speed > runtime efficiency. Can optimize later.

6. **Future Migration**: If needed, can migrate to iced later. UI layer is isolated from browser logic.

### Trade-offs Accepted

- Higher idle CPU usage (~1-2% vs <0.5% for iced)
- Less native look (acceptable for privacy-focused browser with custom identity)
- Limited animations (not critical for MVP)

---

## Implementation Plan

**Milestone 1.2: Basic UI with egui** ✅ Complete
- Tab bar with multiple tabs
- URL bar with text input
- Back/Forward/Refresh buttons
- Settings menu

**Future (Post-MVP)**:
- Consider iced migration if CPU usage becomes issue
- Or optimize egui rendering (request_repaint only on changes)

---

## References

- egui: https://github.com/emilk/egui
- iced: https://github.com/iced-rs/iced
- Comparison: https://areweguiyet.com/
- egui examples: https://www.egui.rs/#demo

---

## Next Steps

1. ✅ Decision documented
2. ✅ Implement basic UI (Milestone 1.2)
3. ⏳ Measure CPU usage and optimize if needed
4. ⏳ Reassess after MVP if performance is concern
