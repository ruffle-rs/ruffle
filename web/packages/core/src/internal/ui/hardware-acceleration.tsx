import { text } from "../i18n";

export function HardwareAcceleration() {
    return (
        <div id="hardware-acceleration-modal" class="modal hidden">
            <div class="modal-area">
                <span class="close-modal"></span>
                <span id="acceleration-text">{ text("enable-hardware-acceleration") }</span>
                <a href="https://github.com/ruffle-rs/ruffle/wiki/Frequently-Asked-Questions-For-Users#chrome-hardware-acceleration" target="_blank" class="modal-button">
                    { text("enable-hardware-acceleration-link") }
                </a>
            </div>
        </div>
    );
}
