import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import St from 'gi://St';

const SHORTCUT_KEY = 'activate-shortcut';
const DESKTOP_ID = 'io.clipl.ClipLinux.desktop';
const RECONNECT_MS = 2000;
const PASTE_DELAY_MS = 120;
const CLIPBOARD_DEBOUNCE_MS = 50;
const MAX_CLIPBOARD_CHARS = 1024 * 1024;

export default class ClipLinuxExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._insertTarget = null;
        this._insertConnection = null;
        this._insertCancellable = new Gio.Cancellable();
        this._clipboard = St.Clipboard.get_default();
        this._lastClipboard = '';
        this._clipboardTimeout = 0;
        this._ownerChangedId = 0;
        this._selection = global.display.get_selection();
        if (this._selection) {
            this._ownerChangedId = this._selection.connect('owner-changed', (_sel, type) => {
                if (type === Meta.SelectionType.SELECTION_CLIPBOARD)
                    this._scheduleClipboardRead();
            });
        }
        Main.wm.addKeybinding(
            SHORTCUT_KEY,
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.ALL,
            () => this._onShortcut(),
        );
        this._runInsertSubscriber();
    }

    disable() {
        Main.wm.removeKeybinding(SHORTCUT_KEY);
        if (this._clipboardTimeout) {
            GLib.source_remove(this._clipboardTimeout);
            this._clipboardTimeout = 0;
        }
        if (this._selection && this._ownerChangedId) {
            this._selection.disconnect(this._ownerChangedId);
            this._ownerChangedId = 0;
        }
        this._selection = null;
        this._clipboard = null;
        this._lastClipboard = '';
        if (this._insertCancellable) {
            this._insertCancellable.cancel();
            this._insertCancellable = null;
        }
        closeQuietly(this._insertConnection);
        this._insertConnection = null;
        this._insertTarget = null;
        this._settings = null;
    }

    _onShortcut() {
        this._rememberFocus();
        const delivered = sendToggle(socketPath());
        tryActivateDesktopApp();
        if (!delivered)
            Main.notify('ClipLinux', 'clipl-daemon is not running. Start it, then press Super+Alt+V again.');
    }

    _scheduleClipboardRead() {
        if (this._clipboardTimeout) {
            GLib.source_remove(this._clipboardTimeout);
            this._clipboardTimeout = 0;
        }
        this._clipboardTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, CLIPBOARD_DEBOUNCE_MS, () => {
            this._clipboardTimeout = 0;
            this._readClipboard();
            return GLib.SOURCE_REMOVE;
        });
    }

    _readClipboard() {
        if (!this._clipboard)
            return;
        this._clipboard.get_text(St.ClipboardType.CLIPBOARD, (_clip, text) => {
            if (!this._clipboard)
                return;
            if (!text || text === this._lastClipboard)
                return;
            if (text.length > MAX_CLIPBOARD_CHARS)
                return;
            this._lastClipboard = text;
            recordClipboardText(text);
        });
    }

    _rememberFocus() {
        const focus = global.display.focus_window;
        if (focus && !isClipLinuxWindow(focus))
            this._insertTarget = focus;
    }

    _runInsertSubscriber() {
        const cancellable = this._insertCancellable;
        this._insertLoop(cancellable).catch(error => {
            if (!cancellable || cancellable.is_cancelled())
                return;
            logError(error, 'ClipLinux insert subscriber stopped');
        });
    }

    async _insertLoop(cancellable) {
        while (cancellable && !cancellable.is_cancelled()) {
            try {
                await this._subscribeInsert(cancellable);
            } catch (error) {
                if (cancellable.is_cancelled())
                    return;
                const message = error && error.message ? String(error.message) : String(error);
                if (!message.includes('socket is missing') && !message.includes('IPC ended'))
                    logError(error, 'ClipLinux SubscribeInsert reconnecting');
            }
            closeQuietly(this._insertConnection);
            this._insertConnection = null;
            if (cancellable.is_cancelled())
                return;
            await delayMs(RECONNECT_MS, cancellable);
        }
    }

    async _subscribeInsert(cancellable) {
        const path = socketPath();
        const file = Gio.File.new_for_path(path);
        if (!file.query_exists(null))
            throw new Error('clipl-daemon socket is missing');

        const client = new Gio.SocketClient();
        const address = new Gio.UnixSocketAddress({path});
        const connection = await connectAsync(client, address, cancellable);
        this._insertConnection = connection;
        const output = connection.get_output_stream();
        const input = connection.get_input_stream();
        writeFrame(output, {
            id: GLib.uuid_string_random(),
            payload: {Request: 'SubscribeInsert'},
        });
        await readFrameAsync(input, cancellable);
        while (!cancellable.is_cancelled()) {
            const envelope = await readFrameAsync(input, cancellable);
            if (isPrepareEvent(envelope))
                this._rememberFocus();
            if (isInsertEvent(envelope))
                this._restoreAndPaste();
        }
    }

    _restoreAndPaste() {
        const win = this._insertTarget;
        if (win && !isClipLinuxWindow(win)) {
            try {
                win.activate(global.get_current_time());
            } catch (error) {
                logError(error, 'ClipLinux could not restore the previous window');
            }
        }
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, PASTE_DELAY_MS, () => {
            try {
                sendCtrlV();
            } catch (error) {
                logError(error, 'ClipLinux Ctrl+V insert failed');
            }
            return GLib.SOURCE_REMOVE;
        });
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
        closeQuietly(connection);
        return false;
    }
}

function recordClipboardText(text) {
    const path = socketPath();
    const file = Gio.File.new_for_path(path);
    if (!file.query_exists(null))
        return;

    let connection = null;
    try {
        const client = new Gio.SocketClient();
        const address = new Gio.UnixSocketAddress({path});
        connection = client.connect(address, null);
        const output = connection.get_output_stream();
        const input = connection.get_input_stream();
        writeFrame(output, {
            id: GLib.uuid_string_random(),
            payload: {Request: {RecordClipboard: {text}}},
        });
        readFrame(input);
        connection.close(null);
    } catch (_error) {
        closeQuietly(connection);
    }
}

function isDelivered(reply) {
    const payload = reply?.payload?.Response;
    if (payload && Object.prototype.hasOwnProperty.call(payload, 'DesktopRouted'))
        return Boolean(payload.DesktopRouted.delivered);
    return payload === 'DesktopRouted' ? false : Boolean(reply);
}

function isInsertEvent(envelope) {
    return envelope?.payload?.Event === 'InsertIntoApp';
}

function isPrepareEvent(envelope) {
    return envelope?.payload?.Event === 'PrepareInsert';
}

function isClipLinuxWindow(win) {
    if (!win)
        return true;
    const bits = [];
    try {
        bits.push(win.get_wm_class() || '');
    } catch (_error) {
        // ignore
    }
    try {
        bits.push(win.get_wm_class_instance() || '');
    } catch (_error) {
        // ignore
    }
    const label = bits.join(' ').toLowerCase();
    return label.includes('clipl');
}

function sendCtrlV() {
    const seat = Clutter.get_default_backend().get_default_seat();
    const device = seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
    const time = global.get_current_time() || Clutter.CURRENT_TIME;
    device.notify_keyval(time, Clutter.KEY_Control_L, Clutter.KeyState.PRESSED);
    device.notify_keyval(time, Clutter.KEY_v, Clutter.KeyState.PRESSED);
    device.notify_keyval(time, Clutter.KEY_v, Clutter.KeyState.RELEASED);
    device.notify_keyval(time, Clutter.KEY_Control_L, Clutter.KeyState.RELEASED);
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

function readFrameAsync(input, cancellable) {
    return readExactAsync(input, 4, cancellable).then(headerBytes => {
        const length = new DataView(headerBytes.buffer).getUint32(0, true);
        if (length > 8 * 1024 * 1024)
            throw new Error('ClipLinux IPC frame too large');
        return readExactAsync(input, length, cancellable);
    }).then(body => JSON.parse(new TextDecoder().decode(body)));
}

function readExactAsync(input, size, cancellable) {
    const out = new Uint8Array(size);
    let offset = 0;

    const readChunk = () => new Promise((resolve, reject) => {
        if (cancellable && cancellable.is_cancelled()) {
            reject(new Error('ClipLinux insert subscriber cancelled'));
            return;
        }
        input.read_bytes_async(size - offset, GLib.PRIORITY_DEFAULT, cancellable, (stream, result) => {
            try {
                const chunk = stream.read_bytes_finish(result);
                if (!chunk || chunk.get_size() === 0) {
                    reject(new Error('ClipLinux IPC ended early'));
                    return;
                }
                const data = new Uint8Array(chunk.get_data());
                out.set(data, offset);
                offset += data.byteLength;
                resolve();
            } catch (error) {
                reject(error);
            }
        });
    });

    const loop = () => {
        if (offset >= size)
            return Promise.resolve(out);
        return readChunk().then(loop);
    };
    return loop();
}

function connectAsync(client, address, cancellable) {
    return new Promise((resolve, reject) => {
        client.connect_async(address, cancellable, (socketClient, result) => {
            try {
                resolve(socketClient.connect_finish(result));
            } catch (error) {
                reject(error);
            }
        });
    });
}

function delayMs(ms, cancellable) {
    return new Promise(resolve => {
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, ms, () => {
            resolve();
            return GLib.SOURCE_REMOVE;
        });
    }).then(() => {
        if (cancellable && cancellable.is_cancelled())
            throw new Error('ClipLinux insert subscriber cancelled');
    });
}

function closeQuietly(connection) {
    if (!connection)
        return;
    try {
        connection.close(null);
    } catch (_error) {
        // ignore
    }
}

function tryActivateDesktopApp() {
    const app = Shell.AppSystem.get_default().lookup_app(DESKTOP_ID);
    if (app)
        app.activate();
}
