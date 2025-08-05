/**
 * @returns The HTMLElement that displays video with an unsupported codec
 */
export function UnsupportedVideo() {
    return (
        <div id="video-modal" class="modal hidden">
            <div class="modal-area">
                <span class="close-modal"></span>
                <div id="video-holder"></div>
            </div>
        </div>
    );
}
