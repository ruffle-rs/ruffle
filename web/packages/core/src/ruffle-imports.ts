/**
 * Functions imported from JS into Ruffle.
 *
 * @ignore
 * @internal
 */

/**
 * Copies data into the given audio channel.
 * This is necessary because Safari does not support `AudioBuffer.copyToChannel`.
 *
 * @internal
 */
export function copyToAudioBuffer(
    audioBuffer: AudioBuffer,
    leftData: ArrayLike<number>,
    rightData: ArrayLike<number>
): void {
    if (leftData) {
        const dstBuffer = audioBuffer.getChannelData(0);
        dstBuffer.set(leftData);
    }

    if (rightData) {
        const dstBuffer = audioBuffer.getChannelData(1);
        dstBuffer.set(rightData);
    }
}

/**
 * Returns the estimated output timestamp for the audio context.
 * This is necessary because web-sys does not export `AudioContext.baseLatency`.
 *
 * @internal
 */
export function getAudioOutputTimestamp(context: AudioContext): number {
    // TODO: Ideally we'd use `context.getOutputTimestamp`, but this is broken as of Safari 15.4.
    return context.currentTime - context.baseLatency;
}
