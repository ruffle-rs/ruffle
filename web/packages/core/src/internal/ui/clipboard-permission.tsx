import { text } from "../i18n";

const shortcutModifier = navigator.userAgent.includes("Mac OS X")
    ? "Command"
    : "Ctrl";

export function ClipboardPermission() {
    return (
        <div id="clipboard-modal" class="modal hidden">
            <div class="modal-area">
                <span class="close-modal"></span>
                <h2>{ text("clipboard-message-title") }</h2>
                <p id="clipboard-modal-description"></p>
                <p>
                    <b>{ shortcutModifier }+C</b>
                    <span>{ text("clipboard-message-copy") }</span>
                </p>
                <p>
                    <b>{ shortcutModifier }+X</b>
                    <span>{ text("clipboard-message-cut") }</span>
                </p>
                <p>
                    <b>{ shortcutModifier }+V</b>
                    <span>{ text("clipboard-message-paste") }</span>
                </p>
            </div>
        </div>
    );
}
