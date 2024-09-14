/**
 * Functions imported from JS into Ruffle.
 *
 * @ignore
 * @internal
 */

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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function callExternalInterface(name: string, args: any[]): any {
    // [NA] Yes, this is indirect eval. Yes, this is a Bad Thing when it comes to security.
    // In fact, yes this is vulnerable to an XSS attack!
    // But plot twist: Flash allowed for this and many games *rely on it*. :(
    // Flash content can do `call("eval", "....")` regardless, this doesn't enable anything that wasn't already permitted.
    // It just goes against what the documentation says, and *looks* really suspicious.
    // Content can only run this if the website has enabled `allowScriptAccess`, so it has to be enabled by the website too.
    return new Function(`return (${name})(...arguments);`)(...args);
}
