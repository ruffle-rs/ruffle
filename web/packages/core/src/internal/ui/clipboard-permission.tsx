const shortcutModifier = navigator.userAgent.includes("Mac OS X")
    ? "Command"
    : "Ctrl";

/**
 * @returns The HTMLElement representing the clipboard permission modal
 */
export function ClipboardPermission() {
    return (
        <div id="clipboard-modal" class="modal hidden">
            <div class="modal-area">
                <span class="close-modal"></span>
                <h2 data-i18n-key="clipboard-message-title"></h2>
                <p id="clipboard-modal-description"></p>
                <p>
                    <b>{shortcutModifier}+C</b>
                    <span data-i18n-key="clipboard-message-copy"></span>
                </p>
                <p>
                    <b>{shortcutModifier}+X</b>
                    <span data-i18n-key="clipboard-message-cut"></span>
                </p>
                <p>
                    <b>{shortcutModifier}+V</b>
                    <span data-i18n-key="clipboard-message-paste"></span>
                </p>
            </div>
        </div>
    );
}
