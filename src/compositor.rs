// SPDX-License-Identifier: Apache-2.0

//! macOS compositor for displaying Linux GUI applications
//!
//! This module provides the host-side compositor that receives graphics
//! from the guest's virtio-gpu device and displays them on macOS.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApplication, NSWindow, NSWindowStyleMask, NSBackingStoreType, NSView};
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil, YES, NO};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSRect, NSPoint, NSSize, NSString, NSAutoreleasePool};
#[cfg(target_os = "macos")]
use objc::runtime::Class;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

/// Configuration for the macOS compositor
#[derive(Clone, Debug)]
pub struct CompositorConfig {
    /// Width of the display window
    pub width: u32,
    
    /// Height of the display window
    pub height: u32,
    
    /// Title for the compositor window
    pub window_title: String,
    
    /// Path to virtio-gpu framebuffer (shared memory)
    pub framebuffer_path: Option<PathBuf>,
}

impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            window_title: "krunkit - Linux GUI".to_string(),
            framebuffer_path: None,
        }
    }
}

impl CompositorConfig {
    /// Create a new compositor configuration
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }
    
    /// Set the window title
    pub fn with_title(mut self, title: String) -> Self {
        self.window_title = title;
        self
    }
    
    /// Set the framebuffer path
    pub fn with_framebuffer(mut self, path: PathBuf) -> Self {
        self.framebuffer_path = Some(path);
        self
    }
}

/// macOS compositor that displays graphics from the guest VM
pub struct Compositor {
    config: CompositorConfig,
    running: Arc<Mutex<bool>>,
}

impl Compositor {
    /// Create a new compositor with the given configuration
    pub fn new(config: CompositorConfig) -> Self {
        Self {
            config,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the compositor in a background thread
    ///
    /// This creates a macOS window and continuously reads from the virtio-gpu
    /// framebuffer to display graphics from the guest VM.
    pub fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(anyhow!("Compositor is already running"));
        }
        *running = true;
        drop(running);
        
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        
        thread::spawn(move || {
            if let Err(e) = Self::compositor_thread(config, running) {
                log::error!("Compositor thread error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Stop the compositor
    pub fn stop(&mut self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// Check if the compositor is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    
    /// Main compositor thread that handles graphics display
    #[cfg(target_os = "macos")]
    fn compositor_thread(config: CompositorConfig, running: Arc<Mutex<bool>>) -> Result<()> {
        log::info!("Starting macOS compositor: {}x{}", config.width, config.height);
        log::info!("Window title: {}", config.window_title);
        
        unsafe {
            // Create autorelease pool for memory management
            let pool = NSAutoreleasePool::new(nil);
            
            // Initialize NSApplication (required for window creation)
            let app = NSApplication::sharedApplication(nil);
            app.setActivationPolicy_(cocoa::appkit::NSApplicationActivationPolicyRegular);
            
            // Create window frame
            let frame = NSRect::new(
                NSPoint::new(100.0, 100.0),
                NSSize::new(config.width as f64, config.height as f64),
            );
            
            // Window style mask: titled, closable, miniaturizable, resizable
            let style_mask = NSWindowStyleMask::NSTitledWindowMask
                | NSWindowStyleMask::NSClosableWindowMask
                | NSWindowStyleMask::NSMiniaturizableWindowMask
                | NSWindowStyleMask::NSResizableWindowMask;
            
            // Create the window
            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                frame,
                style_mask,
                NSBackingStoreType::NSBackingStoreBuffered,
                NO,
            );
            
            if window == nil {
                return Err(anyhow!("Failed to create NSWindow"));
            }
            
            // Set window title
            let title = NSString::alloc(nil).init_str(&config.window_title);
            window.setTitle_(title);
            
            // Center the window on screen
            window.center();
            
            // Create content view for rendering
            let content_view = window.contentView();
            
            // Set background color (dark gray for now, will show framebuffer later)
            let color_class = Class::get("NSColor").ok_or_else(|| anyhow!("NSColor class not found"))?;
            let dark_gray: id = msg_send![color_class, darkGrayColor];
            let _: () = msg_send![content_view, setWantsLayer: YES];
            let layer: id = msg_send![content_view, layer];
            let _: () = msg_send![layer, setBackgroundColor: dark_gray];
            
            // Make window visible
            window.makeKeyAndOrderFront_(nil);
            app.activateIgnoringOtherApps_(YES);
            
            log::info!("Compositor window created and displayed");
            log::info!("Window is now visible on macOS");
            
            // Create a simple framebuffer simulation (placeholder for actual virtio-gpu data)
            // In production, this would read from shared memory
            let mut frame_count: u64 = 0;
            
            // Main event loop - process events and update display
            while *running.lock().unwrap() {
                // Process pending events
                let event_mask = cocoa::appkit::NSAnyEventMask;
                let distant_past: id = msg_send![Class::get("NSDate").unwrap(), distantPast];
                let event: id = msg_send![
                    app,
                    nextEventMatchingMask: event_mask
                    untilDate: distant_past
                    inMode: cocoa::appkit::NSDefaultRunLoopMode
                    dequeue: YES
                ];
                
                if event != nil {
                    let _: () = msg_send![app, sendEvent: event];
                }
                
                // Update frame counter and display
                frame_count += 1;
                if frame_count % 60 == 0 {
                    log::debug!("Compositor: {} frames rendered", frame_count);
                }
                
                // In production: Read from virtio-gpu framebuffer and update window content
                // For now, the window displays with the dark gray background
                
                // Sleep to maintain ~60 FPS
                thread::sleep(Duration::from_millis(16));
                
                // Check if window was closed
                let is_visible: bool = msg_send![window, isVisible];
                if !is_visible {
                    log::info!("Window closed by user, stopping compositor");
                    *running.lock().unwrap() = false;
                    break;
                }
            }
            
            // Cleanup
            let _: () = msg_send![window, close];
            let _: () = msg_send![pool, drain];
            
            log::info!("Compositor thread stopped");
            Ok(())
        }
    }
    
    /// Compositor thread for non-macOS platforms (stub)
    #[cfg(not(target_os = "macos"))]
    fn compositor_thread(_config: CompositorConfig, _running: Arc<Mutex<bool>>) -> Result<()> {
        Err(anyhow!("Compositor is only supported on macOS"))
    }
}

impl Drop for Compositor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Helper function to create and start a compositor for WSLg mode
pub fn create_wslg_compositor(width: u32, height: u32) -> Result<Compositor> {
    let config = CompositorConfig::new(width, height)
        .with_title(format!("krunkit - Linux GUI ({}x{})", width, height));
    
    let mut compositor = Compositor::new(config);
    compositor.start()?;
    
    Ok(compositor)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compositor_config_default() {
        let config = CompositorConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.window_title, "krunkit - Linux GUI");
    }
    
    #[test]
    fn test_compositor_config_new() {
        let config = CompositorConfig::new(800, 600);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }
    
    #[test]
    fn test_compositor_config_with_title() {
        let config = CompositorConfig::default()
            .with_title("Test Window".to_string());
        assert_eq!(config.window_title, "Test Window");
    }
    
    #[test]
    fn test_compositor_creation() {
        let config = CompositorConfig::new(1024, 768);
        let compositor = Compositor::new(config);
        assert!(!compositor.is_running());
    }
}
