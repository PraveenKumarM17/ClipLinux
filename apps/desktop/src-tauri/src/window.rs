//! Picker window visibility. Independent of Tauri so tests need no WebView.

use clipl_core::ActivationRequest;

/// Whether the palette window is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerVisibility {
    /// Window is visible.
    Shown,
    /// Window is hidden; the process is still running.
    Hidden,
}

/// Apply a daemon activation event to local window state.
pub fn apply_activation(current: PickerVisibility, request: ActivationRequest) -> PickerVisibility {
    match request {
        ActivationRequest::ShowPicker => PickerVisibility::Shown,
        ActivationRequest::HidePicker => PickerVisibility::Hidden,
        ActivationRequest::TogglePicker => match current {
            PickerVisibility::Shown => PickerVisibility::Hidden,
            PickerVisibility::Hidden => PickerVisibility::Shown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_is_idempotent() {
        assert_eq!(
            apply_activation(PickerVisibility::Shown, ActivationRequest::ShowPicker),
            PickerVisibility::Shown
        );
        assert_eq!(
            apply_activation(PickerVisibility::Hidden, ActivationRequest::ShowPicker),
            PickerVisibility::Shown
        );
    }

    #[test]
    fn toggle_flips() {
        assert_eq!(
            apply_activation(PickerVisibility::Hidden, ActivationRequest::TogglePicker),
            PickerVisibility::Shown
        );
        assert_eq!(
            apply_activation(PickerVisibility::Shown, ActivationRequest::TogglePicker),
            PickerVisibility::Hidden
        );
    }

    #[test]
    fn hide_does_not_quit() {
        assert_eq!(
            apply_activation(PickerVisibility::Shown, ActivationRequest::HidePicker),
            PickerVisibility::Hidden
        );
    }
}
