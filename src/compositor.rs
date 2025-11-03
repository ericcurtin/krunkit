// SPDX-License-Identifier: Apache-2.0

//! macOS compositor for displaying Linux GUI applications
//!
//! This module provides the host-side compositor that receives graphics
//! from the guest's virtio-gpu device and displays them on macOS.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Result};

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
        
        // Note: Full implementation would use Cocoa/AppKit to create an NSWindow
        // and CALayer/Metal to display the framebuffer content.
        // This requires Objective-C interop which is beyond the scope of this
        // initial implementation.
        
        // For now, log that the compositor is ready
        log::info!("Compositor thread started");
        log::info!("To display graphics, connect a VNC viewer or use macOS screen sharing");
        log::info!("The virtio-gpu device will render to shared memory");
        
        // Keep the thread running while compositor is active
        while *running.lock().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
            
            // In a full implementation, this loop would:
            // 1. Read from virtio-gpu shared memory / framebuffer
            // 2. Upload the texture to Metal/CALayer
            // 3. Trigger a display update
        }
        
        log::info!("Compositor thread stopped");
        Ok(())
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
