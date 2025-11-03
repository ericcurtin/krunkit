# macOS Compositor Implementation

## Overview

The macOS compositor is the host-side component that receives graphics output from the Linux guest VM and displays it in native macOS windows. This document explains how the compositor works and its current implementation status.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Linux Guest VM                         │
│                                                               │
│  ┌──────────────┐        ┌─────────────┐                   │
│  │ GUI App      │───────▶│   Weston    │                   │
│  │ (Firefox)    │        │ Compositor  │                   │
│  └──────────────┘        └─────────────┘                   │
│                                 │                            │
│                                 ▼                            │
│                          ┌─────────────┐                    │
│                          │ virtio-gpu  │                    │
│                          │   Device    │                    │
│                          └─────────────┘                    │
│                                 │                            │
└─────────────────────────────────┼────────────────────────────┘
                                  │ Shared Memory/
                                  │ Framebuffer
┌─────────────────────────────────┼────────────────────────────┐
│                      macOS Host │                            │
│                                 ▼                            │
│                          ┌─────────────┐                    │
│                          │  krunkit    │                    │
│                          │ Compositor  │                    │
│                          └─────────────┘                    │
│                                 │                            │
│                                 ▼                            │
│                          ┌─────────────┐                    │
│                          │   NSWindow  │                    │
│                          │  CALayer/   │                    │
│                          │   Metal     │                    │
│                          └─────────────┘                    │
│                                 │                            │
│                                 ▼                            │
│                          ┌─────────────┐                    │
│                          │   macOS     │                    │
│                          │ WindowServer│                    │
│                          └─────────────┘                    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## How It Works

### 1. Initialization

When krunkit starts with `--wslg-gui`, the compositor is initialized:

```rust
let compositor = create_wslg_compositor(width, height)?;
```

This creates:
- A compositor configuration with display dimensions
- A background thread for handling graphics updates
- State tracking for the compositor lifecycle

### 2. Graphics Pipeline

**Guest Side (Linux VM):**
1. GUI application renders to Wayland/X11
2. Weston compositor composites the application windows
3. Output is rendered via virtio-gpu device
4. Graphics data is written to shared memory framebuffer

**Host Side (macOS):**
1. Compositor reads from shared memory framebuffer (~60 FPS)
2. Texture data is uploaded to Metal/CALayer
3. macOS window is updated with new frame
4. WindowServer displays the result on screen

### 3. Window Management

The compositor creates native macOS windows that:
- Have standard macOS window chrome (title bar, close button, etc.)
- Support resizing and moving
- Integrate with Mission Control and Spaces
- Can be minimized to the Dock

### 4. Input Handling

User input events (keyboard, mouse) are:
1. Captured by the macOS window
2. Translated to appropriate Linux input events
3. Forwarded to the guest VM via virtio-input devices
4. Processed by the Linux GUI application

## Implementation Status

### ✅ Implemented

- **Compositor Framework**: Core compositor structure and lifecycle management
- **Threading Model**: Background thread for graphics updates
- **Configuration**: Flexible compositor configuration with defaults
- **Integration**: Automatic initialization when `--wslg-gui` is enabled
- **Logging**: Comprehensive logging for debugging

### 🚧 In Progress

- **Cocoa/AppKit Integration**: Native macOS window creation
  - Requires Objective-C/Swift interop
  - Need to bridge Rust with NSWindow APIs
  
- **Metal/CALayer Rendering**: GPU-accelerated display
  - Metal texture creation and updates
  - CALayer integration for efficient compositing
  
- **Framebuffer Access**: Reading virtio-gpu shared memory
  - Interface with libkrun-efi framebuffer API
  - Memory mapping and synchronization

### 📋 Planned

- **Input Forwarding**: Complete keyboard and mouse event forwarding
- **Clipboard Integration**: Copy/paste between macOS and Linux
- **Multi-Monitor Support**: Display across multiple screens
- **Window Decorations**: Custom or native window chrome options
- **Performance Optimization**: Frame timing and vsync

## Current Usage

In the current implementation, when you start krunkit with `--wslg-gui`:

```bash
krunkit --cpus 4 --memory 4096 --wslg-gui \
  --device virtio-blk,path=ubuntu.img,format=raw
```

The compositor will:
1. ✅ Initialize successfully
2. ✅ Start a background thread
3. ✅ Log its status and configuration
4. 🚧 Display graphics via alternative methods (see below)

## Viewing Graphics (Current Options)

While the full native window integration is being implemented, you can view graphics using:

### Option 1: VNC

Connect to the VM with a VNC client:
```bash
# From macOS
open vnc://localhost:5900
```

### Option 2: macOS Screen Sharing

Use built-in macOS Screen Sharing to connect to the guest display.

### Option 3: X11 Forwarding

Forward X11 over SSH for individual applications:
```bash
ssh -X user@vm-ip
firefox &
```

## Development Roadmap

### Phase 1: Core Window Display (Current)
- [x] Compositor framework and threading
- [ ] Basic NSWindow creation
- [ ] Framebuffer display in window
- [ ] 60 FPS refresh loop

### Phase 2: Input and Interaction
- [ ] Keyboard event forwarding
- [ ] Mouse event forwarding
- [ ] Window focus management
- [ ] Window resizing

### Phase 3: Advanced Features
- [ ] Clipboard integration
- [ ] Multi-monitor support
- [ ] Native macOS look and feel
- [ ] Performance optimizations

### Phase 4: Polish
- [ ] Automatic window positioning
- [ ] Window shadows and effects
- [ ] Mission Control integration
- [ ] Dock icon and menu

## Technical Implementation Notes

### Objective-C Interop

Full implementation requires Objective-C interop for Cocoa/AppKit:

```rust
// Future implementation using objc crate
#[cfg(target_os = "macos")]
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};

// Create NSWindow
let window: *mut Object = msg_send![class!(NSWindow), alloc];
let window: *mut Object = msg_send![window, 
    initWithContentRect:frame
    styleMask:style
    backing:backing
    defer:NO
];
```

### Metal Rendering

Graphics rendering will use Metal for GPU acceleration:

```rust
use metal::*;

let device = Device::system_default().unwrap();
let command_queue = device.new_command_queue();
let texture = device.new_texture(&texture_descriptor);

// Update texture with framebuffer data
// Present to CALayer
```

### Shared Memory Access

Access to virtio-gpu framebuffer requires libkrun-efi API:

```rust
extern "C" {
    fn krun_get_framebuffer(ctx_id: u32) -> *mut u8;
    fn krun_get_framebuffer_size(ctx_id: u32) -> usize;
}

// Map framebuffer to Rust slice
let fb_ptr = unsafe { krun_get_framebuffer(ctx_id) };
let fb_size = unsafe { krun_get_framebuffer_size(ctx_id) };
let framebuffer = unsafe { 
    std::slice::from_raw_parts(fb_ptr, fb_size) 
};
```

## Performance Considerations

### Target Performance Metrics

- **Frame Rate**: 60 FPS for smooth display
- **Input Latency**: < 16ms for responsive feel
- **Memory Usage**: Minimal overhead beyond framebuffer
- **CPU Usage**: < 5% for compositor thread

### Optimization Strategies

1. **Frame Timing**: Skip updates if framebuffer unchanged
2. **Partial Updates**: Only update changed regions
3. **Double Buffering**: Prevent tearing and flicker
4. **GPU Acceleration**: Offload compositing to GPU

## Testing

### Unit Tests

Run compositor unit tests:
```bash
cargo test compositor
```

### Integration Tests

Test with actual VM:
```bash
# Start with compositor enabled
krunkit --wslg-gui --cpus 2 --memory 2048 \
  --device virtio-blk,path=test.img,format=raw

# Check logs for compositor initialization
tail -f /tmp/krunkit.log | grep compositor
```

### Performance Tests

Measure compositor performance:
```bash
# Monitor CPU usage
top | grep krunkit

# Check frame timing
# (requires instrumentation in compositor code)
```

## Troubleshooting

### Compositor Doesn't Start

Check logs for initialization errors:
```bash
tail -f /tmp/krunkit.log | grep -i compositor
```

Common issues:
- Missing macOS SDK or XCode tools
- Insufficient permissions
- Conflicting display settings

### Poor Performance

If compositor is slow:
1. Reduce resolution: `--wslg-gpu-width 1280 --wslg-gpu-height 720`
2. Check CPU/GPU load
3. Verify GPU acceleration is enabled
4. Check for resource contention

### Graphics Not Appearing

Verify configuration:
1. Confirm `--wslg-gui` flag is set
2. Check virtio-gpu device is configured
3. Verify Weston is running in guest
4. Review compositor logs

## Contributing

To contribute to compositor development:

1. **Objective-C/Swift Experience**: Help with Cocoa/AppKit integration
2. **Metal Expertise**: Implement GPU-accelerated rendering
3. **macOS Development**: Native window management and features
4. **Testing**: Platform compatibility and performance testing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## References

- [Apple NSWindow Documentation](https://developer.apple.com/documentation/appkit/nswindow)
- [Metal Framework](https://developer.apple.com/metal/)
- [virtio-gpu Specification](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [WSLg Architecture](https://github.com/microsoft/wslg)
