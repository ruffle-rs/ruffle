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

/**
 * Copies interleaved stereo audio data into an `AudioBuffer`.
 *
 * @internal
 */
export function copyToAudioBufferInterleaved(
    audioBuffer: AudioBuffer,
    interleavedData: ArrayLike<number>
): void {
    const numSamples = audioBuffer.length;
    const leftBuffer = audioBuffer.getChannelData(0);
    const rightBuffer = audioBuffer.getChannelData(1);
    let i = 0;
    let sample = 0;
    while (sample < numSamples) {
        leftBuffer[sample] = interleavedData[i];
        rightBuffer[sample] = interleavedData[i + 1];
        sample++;
        i += 2;
    }
}
