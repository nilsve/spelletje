#!/usr/bin/env python3
"""Integration test runner for spelletje-mac (macroquad game on XWayland).

Launches the game, takes screenshots every second, and sends keyboard inputs.

Usage:
    # Build and run with default sequence (hold A for 3s, release, hold W for 1s)
    python test_runner.py

    # Run the built binary instead of cargo run
    python test_runner.py --binary

    # Custom key sequence: hold 'a' for 2s, release for 0.5s, hold 'w' for 1s, repeat 3 times
    python test_runner.py --keys "a:2,w:1" --repeat 3

    # Screenshot interval in seconds
    python test_runner.py --interval 0.5

    # Output directory for screenshots
    python test_runner.py --output ./screenshots

    # Run only the key sequence (no screenshots, game must already be running)
    python test_runner.py --run-sequence --keys "a:3,w:2"

    # Run only screenshots for 5 seconds (game must already be running)
    python test_runner.py --capture-only --duration 5

    # Kill any existing test_runner-spelletje game instances
    python test_runner.py --kill

Requirements:
    - xdotool (for sending keyboard input to XWayland window)
    - import (ImageMagick) or scrot or gnome-screenshot (for screenshots)
    - cargo or the built binary
"""

import argparse
import subprocess
import time
import os
import sys
import glob
from pathlib import Path
from datetime import datetime


WINDOW_SEARCH = "Spelletje"
SCRIPT_DIR = Path(__file__).parent


def find_x_display():
    """Find an active X11 display socket."""
    x_sockets = Path("/tmp/.X11-unix")
    if x_sockets.exists():
        for sock in sorted(x_sockets.iterdir()):
            display_num = int(sock.name.replace("X", ""))
            env = os.environ.copy()
            env["DISPLAY"] = f":{display_num}"
            result = subprocess.run(
                ["xdotool", "getactivewindow"],
                capture_output=True, text=True, timeout=2, env=env
            )
            if result.returncode == 0:
                print(f"  Using X display :{display_num}")
                return f":{display_num}"
    # Fallback: try :0
    return ":0"


def find_wayland_display():
    """Find the Wayland display socket."""
    runtime = Path(f"/run/user/{os.getuid()}")
    if runtime.exists():
        for sock in runtime.glob("wayland-*"):
            return str(sock)
    return None


def get_env():
    """Get environment with display vars set."""
    env = os.environ.copy()
    display = find_x_display()
    env["DISPLAY"] = display
    wayland = find_wayland_display()
    if wayland:
        env["WAYLAND_DISPLAY"] = "wayland-0"
    # Xauthority for auth
    xauth = Path.home() / ".Xauthority"
    if xauth.exists():
        env["XAUTHORITY"] = str(xauth)
    return env


def find_capture_tool():
    for tool in ["import", "scrot", "gnome-screenshot"]:
        if shutil.which(tool):
            return tool
    return None


def capture_screenshot(path):
    tool = find_capture_tool()
    if not tool:
        print("ERROR: No screenshot tool found (need import/scrot/gnome-screenshot)")
        return False

    env = get_env()
    if tool == "import":
        cmd = ["import", "-window", WINDOW_SEARCH, path]
    elif tool == "scrot":
        cmd = ["scrot", "--window", WINDOW_SEARCH, path]
    else:
        cmd = ["gnome-screenshot", "--window", "--file", path, "--window-focus-delay=1"]

    try:
        subprocess.run(cmd, check=True, capture_output=True, timeout=5, env=env)
        return os.path.exists(path)
    except subprocess.TimeoutExpired:
        print(f"  screenshot timeout")
        return False
    except subprocess.CalledProcessError as e:
        print(f"  screenshot failed: {e.stderr.decode().strip()}")
        return False


def xdotool_window_id():
    """Find the X11 window ID for our game window."""
    env = get_env()
    try:
        result = subprocess.run(
            ["xdotool", "search", "--name", WINDOW_SEARCH],
            capture_output=True, text=True, timeout=3, env=env
        )
        ids = result.stdout.strip().split("\n")
        if ids and ids[0]:
            return ids[0]
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return None


def send_key(window_id, key, duration=None):
    """Send a key press to the game window. If duration is set, hold the key."""
    if not window_id:
        return

    env = get_env()
    if duration is not None:
        try:
            subprocess.run(
                ["xdotool", "keydown", "--window", window_id, key],
                check=True, capture_output=True, timeout=5, env=env
            )
            time.sleep(duration)
            subprocess.run(
                ["xdotool", "keyup", "--window", window_id, key],
                check=True, capture_output=True, timeout=5, env=env
            )
        except (subprocess.TimeoutExpired, subprocess.CalledProcessError):
            pass
    else:
        try:
            subprocess.run(
                ["xdotool", "key", "--window", window_id, key],
                check=True, capture_output=True, timeout=5, env=env
            )
        except (subprocess.TimeoutExpired, subprocess.CalledProcessError):
            pass


def parse_keys(keys_str):
    """Parse key sequence string like 'a:2,w:1,d:0.5'. Returns list of (key, duration)."""
    result = []
    for part in keys_str.split(","):
        part = part.strip()
        if ":" in part:
            key, dur = part.rsplit(":", 1)
            result.append((key.strip(), float(dur.strip())))
        else:
            result.append((part, None))
    return result


def kill_existing():
    """Kill any existing spelletje-mac instances launched by this script."""
    try:
        result = subprocess.run(
            ["pgrep", "-f", "spelletje-mac"],
            capture_output=True, text=True, timeout=3
        )
        pids = result.stdout.strip().split("\n")
        if pids and pids[0]:
            for pid in pids:
                if pid.strip():
                    try:
                        subprocess.run(["kill", "-TERM", pid.strip()], capture_output=True)
                    except Exception:
                        pass
            time.sleep(1)
            for pid in pids:
                if pid.strip():
                    try:
                        subprocess.run(["kill", "-9", pid.strip()], capture_output=True)
                    except Exception:
                        pass
            print(f"Killed existing spelletje-mac processes")
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass


def wait_for_window(timeout=15):
    """Wait for the game window to appear."""
    start = time.time()
    while time.time() - start < timeout:
        wid = xdotool_window_id()
        if wid:
            return wid
        time.sleep(0.3)
    return None


def run_test(args):
    if args.kill:
        kill_existing()
        return

    # Resolve key sequence
    if args.run_sequence and args.keys:
        keys = parse_keys(args.keys)
        window_id = xdotool_window_id()
        if not window_id:
            print(f"ERROR: No window matching '{WINDOW_SEARCH}' found. "
                  f"Start the game first or omit --run-sequence.")
            sys.exit(1)
        for key, dur in keys:
            print(f"  sending '{key}'" + (f" for {dur}s" if dur else ""))
            send_key(window_id, key, dur)
            if dur is None:
                time.sleep(args.key_delay)
        return

    # Launch game
    print("Launching spelletje-mac...")
    if args.binary:
        binary = str(SCRIPT_DIR / "target/debug/spelletje-mac")
        if not os.path.exists(binary):
            print(f"ERROR: Binary not found at {binary}. Run 'cargo build' first.")
            sys.exit(1)
        cmd = [binary]
    else:
        cmd = ["cargo", "run", "--quiet"]

    proc = subprocess.Popen(
        cmd,
        cwd=str(SCRIPT_DIR),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    print(f"  PID: {proc.pid}")

    # Wait for window
    print("Waiting for game window...")
    window_id = wait_for_window()
    if not window_id:
        print("ERROR: Game window did not appear. Is XWayland working?")
        proc.terminate()
        sys.exit(1)
    print(f"  Window found: {window_id}")

    # Setup output dir
    if args.capture_only or args.run_sequence:
        output_dir = Path(args.output)
    else:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_dir = Path(args.output) / ts
        output_dir.mkdir(parents=True, exist_ok=True)
    print(f"  Screenshots -> {output_dir}")

    if args.capture_only:
        # Just capture for --duration seconds
        end = time.time() + args.duration
        count = 0
        while time.time() < end:
            count += 1
            path = output_dir / f"screenshot_{count:04d}.png"
            print(f"  screenshot {count}: {path}", end=" - ")
            if capture_screenshot(str(path)):
                print("ok")
            else:
                print("FAILED")
            time.sleep(args.interval)
        print(f"\nCaptured {count} screenshots")
        return

    # Full test: key sequence + screenshots
    if not args.keys:
        # Default sequence: hold A for 3s, release, hold W for 1s
        keys = parse_keys("a:3,w:1")
    else:
        keys = parse_keys(args.keys)

    repeat = args.repeat if args.repeat else 1
    total = len(keys) * repeat

    print(f"\nRunning {total} key action(s) over {total * 0.1:.1f}s "
          f"(interval={args.interval}s)...")

    screenshot_count = 0
    action_idx = 0
    key_idx = 0

    while action_idx < total:
        key, dur = keys[key_idx % len(keys)]

        # Take screenshot before key action
        screenshot_count += 1
        path = output_dir / f"screenshot_{screenshot_count:04d}.png"
        print(f"  [{action_idx+1}/{total}] screenshot {screenshot_count}: "
              f"{path.name}", end=" - ")
        if capture_screenshot(str(path)):
            print("ok")
        else:
            print("FAILED")

        # Send key
        print(f"  [{action_idx+1}/{total}] key '{key}'" +
              (f" for {dur}s" if dur else ""), end="")
        send_key(window_id, key, dur)
        print()

        action_idx += 1
        key_idx += 1

        # Wait between actions
        if action_idx < total:
            time.sleep(args.interval)

    print(f"\nDone. {screenshot_count} screenshots in {output_dir}")

    if not args.no_wait:
        print("Press Enter to close the game...")
        input()
        proc.terminate()
        proc.wait()
        print("Game closed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Integration test runner for spelletje-mac"
    )
    parser.add_argument("--binary", action="store_true",
                        help="Use built binary instead of cargo run")
    parser.add_argument("--output", default="./screenshots",
                        help="Screenshot output directory (default: ./screenshots)")
    parser.add_argument("--interval", type=float, default=1.0,
                        help="Seconds between screenshots (default: 1.0)")
    parser.add_argument("--keys",
                        help="Key sequence, e.g. 'a:2,w:1,d:0.5' (hold a 2s, release, hold w 1s)")
    parser.add_argument("--repeat", type=int,
                        help="Repeat the key sequence N times (default: 1)")
    parser.add_argument("--run-sequence", action="store_true",
                        help="Only run key sequence (game must be running)")
    parser.add_argument("--capture-only", action="store_true",
                        help="Only capture screenshots for --duration seconds")
    parser.add_argument("--duration", type=float, default=10.0,
                        help="Duration for --capture-only (default: 10s)")
    parser.add_argument("--key-delay", type=float, default=0.2,
                        help="Delay between single key presses (default: 0.2)")
    parser.add_argument("--kill", action="store_true",
                        help="Kill existing spelletje-mac processes")
    parser.add_argument("--no-wait", action="store_true",
                        help="Don't wait for Enter before closing the game")
    args = parser.parse_args()

    import shutil
    run_test(args)
