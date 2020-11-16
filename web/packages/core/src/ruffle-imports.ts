/**
 * Functions imported from JS into Ruffle.
 *
 * @ignore
 */

/**
 * Copies data into the given audio channel.
 * This is necessary because Safari does not support `AudioBuffer.copyToChannel`.
 * @internal
 */
export function copyToAudioBuffer(
    audio_buffer: AudioBuffer,
    left_data: ArrayLike<number>,
    right_data: ArrayLike<number>
): void {
    if (left_data) {
        const dst_buffer = audio_buffer.getChannelData(0);
        dst_buffer.set(left_data);
    }

    if (right_data) {
        const dst_buffer = audio_buffer.getChannelData(1);
        dst_buffer.set(right_data);
    }
}
