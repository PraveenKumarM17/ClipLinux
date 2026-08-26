import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';

const SHORTCUT_KEY = 'activate-shortcut';
const DESKTOP_ID = 'io.clipl.ClipLinux.desktop';

export default class ClipLinuxExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        Main.wm.addKeybinding(
            SHORTCUT_KEY,
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.ALL,
            () => this._onShortcut(),
        );
    }

    disable() {
        Main.wm.removeKeybinding(SHORTCUT_KEY);
        this._settings = null;
    }

    _onShortcut() {
        const delivered = sendToggle(socketPath());
        tryActivateDesktopApp();
        if (!delivered)
            Main.notify('ClipLinux', 'clipl-daemon is not running. Start it, then press Super+V again.');
    }
}

function socketPath() {
    const override = GLib.getenv('CLIPL_RUNTIME_DIR');
    if (override)
        return GLib.build_filenamev([override, 'daemon.sock']);
    const runtime = GLib.getenv('XDG_RUNTIME_DIR') || GLib.get_user_runtime_dir();
    return GLib.build_filenamev([runtime, 'clipl', 'daemon.sock']);
}

function sendToggle(path) {
    const file = Gio.File.new_for_path(path);
    if (!file.query_exists(null))
        return false;

    let connection = null;
    try {
        const client = new Gio.SocketClient();
        const address = new Gio.UnixSocketAddress({path});
        connection = client.connect(address, null);
        const output = connection.get_output_stream();
        const input = connection.get_input_stream();
        writeFrame(output, {
            id: GLib.uuid_string_random(),
            payload: {Request: 'ToggleDesktop'},
        });
        const reply = readFrame(input);
        connection.close(null);
        return isDelivered(reply);
    } catch (error) {
        logError(error, 'ClipLinux ToggleDesktop failed');
        if (connection) {
            try {
                connection.close(null);
            } catch (_closeErr) {
                // ignore
            }
        }
        return false;
    }
}

function isDelivered(reply) {
    const payload = reply?.payload?.Response;
    if (payload && Object.prototype.hasOwnProperty.call(payload, 'DesktopRouted'))
        return Boolean(payload.DesktopRouted.delivered);
    return payload === 'DesktopRouted' ? false : Boolean(reply);
}

function writeFrame(output, obj) {
    const json = JSON.stringify(obj);
    const body = new TextEncoder().encode(json);
    const header = new Uint8Array(4);
    new DataView(header.buffer).setUint32(0, body.byteLength, true);
    output.write_all(header, null);
    output.write_all(body, null);
}

function readFrame(input) {
    const headerBytes = readExact(input, 4);
    const length = new DataView(headerBytes.buffer).getUint32(0, true);
    if (length > 8 * 1024 * 1024)
        throw new Error('ClipLinux IPC frame too large');
    const body = readExact(input, length);
    return JSON.parse(new TextDecoder().decode(body));
}

function readExact(input, size) {
    const out = new Uint8Array(size);
    let offset = 0;
    while (offset < size) {
        const chunk = input.read_bytes(size - offset, null);
        if (!chunk || chunk.get_size() === 0)
            throw new Error('ClipLinux IPC ended early');
        const data = new Uint8Array(chunk.get_data());
        out.set(data, offset);
        offset += data.byteLength;
    }
    return out;
}

function tryActivateDesktopApp() {
    const app = Shell.AppSystem.get_default().lookup_app(DESKTOP_ID);
    if (app)
        app.activate();
}
