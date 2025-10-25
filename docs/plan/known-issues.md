# Known Issues and Limitations

**Milestone 1.3** - Servo Integration
**Status**: Fully operational with minor warnings

---

## Non-Critical Issues (MVP Acceptable)

### 1. Servo ResourceReader Warnings
**Severity**: Low
**Impact**: Console warnings only, no functional impact

```
ERROR embedder_traits::resources: Resource reader not set. (4×)
ERROR net::hsts: HSTS preload file is invalid. (1×)
```

**Root Cause**: ResourceReader trait not implemented
**Workaround**: Servo uses built-in defaults
**MVP Decision**: Skip implementation - not required for basic rendering
**Future**: Implement when custom resources needed (fonts, HSTS data)

### 2. CPU-Based Rendering
**Severity**: Low (performance)
**Impact**: Slower rendering than GPU-accelerated

**Root Cause**: Using SoftwareRenderingContext instead of hardware compositor
**MVP Decision**: Acceptable - simpler implementation, reduces GPU complexity
**Future**: Migrate to hardware compositor for better performance

### 3. Dead Code Warnings
**Severity**: None
**Impact**: Build warnings only

```rust
// delegate.rs:75 - state() method never used
pub fn state(&self) -> &DelegateState {
    &self.state
}
```

**Fix**: Add `#[allow(dead_code)]` or remove unused methods
**Future**: Clean up after API stabilizes

---

## Resolved Issues

### ✅ egui_glow GL Errors (FIXED)
**Was**: 100+ GL errors per frame causing console spam
**Fix**: Switched from egui_glow to egui_wgpu backend
**Commit**: f5f556a

### ✅ Rustls CryptoProvider Panic (FIXED)
**Was**: ResourceManager thread panic on network requests
**Fix**: Initialize rustls crypto provider in main.rs
**Commit**: 8d30ae9

### ✅ Visual Rendering Missing (FIXED - Phase 3)
**Was**: Pixel data retrieved but not displayed (only metadata shown)
**Fix**: Implemented egui ColorImage conversion and texture display
**Implementation**:
- Added `frame_texture: Option<egui::TextureHandle>` to BrowserApp state
- Convert `RenderedFrame` pixels to `ColorImage::from_rgba_unmultiplied()`
- Load texture with `ctx.load_texture()` every frame
- Display with `ui.image()` with aspect ratio preservation
**Commit**: [Phase 3 commit]

---

## Testing Matrix

| Test Case | Status | Notes |
|-----------|--------|-------|
| Launch browser | ✅ Pass | Clean startup |
| Load hardcoded HTML | ✅ Pass | Basic rendering works |
| Load real URL (example.com) | ✅ Pass | Network + rendering works |
| Delegate callbacks | ✅ Pass | Title/URL/load events work |
| egui rendering | ✅ Pass | No GL errors with wgpu |
| **Visual pixel rendering** | ✅ Pass | Pixels displayed as texture |
| Resource loading | ⚠️ Warnings | Non-blocking warnings only |

---

**Last Updated**: 2025-10-25
**Milestone**: 1.3 - Servo Integration COMPLETE (with visual rendering)
