/**
 * Functions improted from JS into Ruffle.
 */

/**
 * Copies data into the given audio channel.
 * This is necessary because Safari does not support `AudioBuffer.copyToChannel`.
 */
export function copy_to_audio_buffer(
    audio_buffer: AudioBuffer,
    left_data: ArrayLike<number>,
    right_data: ArrayLike<number>
) {
    if (left_data) {
        const dst_buffer = audio_buffer.getChannelData(0);
        dst_buffer.set(left_data);
    }

    if (right_data) {
        const dst_buffer = audio_buffer.getChannelData(1);
        dst_buffer.set(right_data);
    }
}
