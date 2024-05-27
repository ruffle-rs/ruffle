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
    rightData: ArrayLike<number>,
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
    interleavedData: ArrayLike<number>,
): void {
    const numSamples = audioBuffer.length;
    const leftBuffer = audioBuffer.getChannelData(0);
    const rightBuffer = audioBuffer.getChannelData(1);
    let i = 0;
    let sample = 0;
    while (sample < numSamples) {
        leftBuffer[sample] = interleavedData[i]!;
        rightBuffer[sample] = interleavedData[i + 1]!;
        sample++;
        i += 2;
    }
}

/**
 * Performs the ActionScript `ExternalInterface.call(name, ...values)`
 *
 * @internal
 */
// @ts-expect-error defined but not used
// eslint-disable-next-line @typescript-eslint/no-explicit-any,@typescript-eslint/no-unused-vars
export function callExternalInterface(name: string, args: any[]): any {
    // [NA] Yes, this is direct eval. Yes, this is a Bad Thing when it comes to security.
    // In fact, yes this is vulnerable to an XSS attack!
    // But plot twist: Flash allowed for this and many content *relies on it*. :(
    return eval(`(${name})(...args)`);
}
