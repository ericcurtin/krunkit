# WSLg-like Features for macOS

## Overview

krunkit now includes WSLg-like functionality for macOS, enabling Linux GUI applications to run seamlessly on macOS hosts with full desktop integration. This implementation uses Wayland, Weston, and PulseAudio to provide a native-feeling experience.

## Architecture

### System Distro

The system distro is a containerized Linux environment where the Weston Wayland compositor and PulseAudio server run. This is similar to WSLg's approach but adapted for macOS:

- **Weston**: Wayland compositor that handles GUI application rendering
- **PulseAudio**: Audio server for microphone and speaker support
- **virtio-gpu**: GPU acceleration for graphics rendering
- **virtio-fs**: File system sharing between macOS and Linux

### Components

1. **Wayland/Weston Compositor**: Manages GUI windows and graphics
2. **PulseAudio Server**: Handles audio input/output
3. **Application Discovery**: Finds installed Linux GUI applications via desktop files
4. **macOS Integration**: Creates native-feeling launchers and shortcuts

## Configuration

### Enabling WSLg-like Features

To enable GUI support, you need to configure several virtio devices:

```bash
krunkit \
  --cpus 4 \
  --memory 4096 \
  --device virtio-blk,path=/path/to/linux.img,format=raw \
  --device virtio-gpu,width=1920,height=1080 \
  --device virtio-input,keyboard \
  --device virtio-input,pointing \
  --device virtio-fs,sharedDir=/tmp/wslg-sockets,mountTag=wslg \
  --device virtio-vsock,port=6000,socketURL=/tmp/wayland.sock,listen \
  --device virtio-vsock,port=4713,socketURL=/tmp/pulseaudio.sock,listen \
  --device virtio-net,type=unixgram,path=/tmp/network.sock,mac=52:54:00:12:34:56
```

### Inside the Linux Guest

The system distro should automatically start:
- Weston compositor on Wayland socket (forwarded via vsock port 6000)
- PulseAudio server on TCP/Unix socket (forwarded via vsock port 4713)

Environment variables are set automatically:
```bash
export WAYLAND_DISPLAY=wayland-0
export PULSE_SERVER=unix:/tmp/pulseaudio.sock
```

## Usage

### Running GUI Applications

Once configured, Linux GUI applications can be launched normally:

```bash
# Launch graphical text editor
gedit myfile.txt

# Launch web browser
firefox

# Launch file manager
nautilus

# Launch GIMP
gimp
```

### Installing GUI Applications

Use your distribution's package manager:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install gedit gimp firefox nautilus

# Fedora
sudo dnf install gedit gimp firefox nautilus

# Arch Linux
sudo pacman -S gedit gimp firefox nautilus
```

## Features

### Supported

- ✅ GUI application rendering via Wayland/Weston
- ✅ GPU acceleration via virtio-gpu
- ✅ Audio output via PulseAudio
- ✅ Audio input via PulseAudio
- ✅ Keyboard input
- ✅ Mouse/pointing input
- ✅ File system sharing via virtio-fs
- ✅ Network connectivity

### Planned

- 🚧 Automatic application discovery
- 🚧 macOS dock/application menu integration
- 🚧 Clipboard sharing between macOS and Linux
- 🚧 Multi-monitor support
- 🚧 Native macOS window chrome

## Technical Details

### Wayland/Weston Setup

The Weston compositor is configured to run in headless mode and forward rendering through the virtio-gpu device. Graphics are then composited by the macOS host.

### PulseAudio Configuration

PulseAudio is configured as a system-wide daemon accessible via Unix socket or TCP. The socket is forwarded from the guest to the host via virtio-vsock.

### GPU Acceleration

GPU acceleration is provided through:
- virtio-gpu device with Venus (Vulkan) support
- virglrenderer for OpenGL support
- Metal translation layer on macOS host

## Troubleshooting

### No GUI Applications Appear

1. Check that virtio-gpu device is configured
2. Verify Weston is running: `ps aux | grep weston`
3. Check WAYLAND_DISPLAY environment variable: `echo $WAYLAND_DISPLAY`

### No Audio

1. Verify PulseAudio is running: `ps aux | grep pulseaudio`
2. Check PULSE_SERVER environment variable: `echo $PULSE_SERVER`
3. Test audio: `paplay /usr/share/sounds/alsa/Front_Center.wav`

### Poor Performance

1. Ensure sufficient memory is allocated (4GB+ recommended)
2. Verify GPU acceleration is enabled
3. Check CPU allocation (4+ vCPUs recommended)

## Differences from WSLg

While inspired by WSLg, this implementation has some key differences:

1. **Display Protocol**: Uses native Wayland/Weston instead of RDP
2. **Window Management**: Relies on virtio-gpu compositing instead of RAIL/VAIL
3. **Integration**: macOS-specific integration instead of Windows Start Menu
4. **Audio**: Direct PulseAudio forwarding instead of RDP audio channels

## Building System Distro

For advanced users who want to customize the system distro, see [CONTRIBUTING.md](../CONTRIBUTING.md) for build instructions.
