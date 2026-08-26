import Adw from 'gi://Adw';
import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class ClipLinuxPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();
        const page = new Adw.PreferencesPage({
            title: 'ClipLinux',
        });
        const group = new Adw.PreferencesGroup({
            title: 'Activation',
            description:
                'On GNOME Wayland the Shell owns this shortcut. ClipLinux does not grab keys from the daemon. Keep the binding in sync with config.toml [activation] if you also use X11.',
        });

        const current = formatBinding(settings.get_strv('activate-shortcut'));
        const row = new Adw.ActionRow({
            title: 'Picker shortcut',
            subtitle:
                `${current}. Change with: gsettings set org.gnome.shell.extensions.clipl activate-shortcut "['<Super>v']"`,
        });
        group.add(row);
        page.add(group);
        window.add(page);
    }
}

function formatBinding(values) {
    if (!values || values.length === 0)
        return 'unset';
    return values.join(', ');
}
