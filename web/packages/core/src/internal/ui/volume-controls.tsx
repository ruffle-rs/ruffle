import { text } from "../i18n";

/**
 * @returns The HTMLElement that can be used to modify the SWF volume
 */
export function VolumeControls() {
    return (
        <div id="volume-controls-modal" class="modal hidden">
            <div class="modal-area">
                <div id="volume-controls">
                    <input id="mute-checkbox" type="checkbox" />
                    <label
                        id="volume-mute"
                        for="mute-checkbox"
                        title={text("volume-controls-unmute")}
                    ></label>
                    <label
                        id="volume-min"
                        for="mute-checkbox"
                        title={text("volume-controls-mute")}
                    ></label>
                    <label
                        id="volume-mid"
                        for="mute-checkbox"
                        title={text("volume-controls-mute")}
                    ></label>
                    <label
                        id="volume-max"
                        for="mute-checkbox"
                        title={text("volume-controls-mute")}
                    ></label>
                    <input
                        id="volume-slider"
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                    />
                    <span id="volume-slider-text"></span>
                    <span class="close-modal"></span>
                </div>
            </div>
        </div>
    );
}
