/**
 * Functions improted from JS into Ruffle.
 */

/**
 * Copies data into the given audio channel.
 * This is necessary because Safari does not support `AudioBuffer.copyToChannel`.
 */
export function copy_to_audio_buffer(audio_buffer, left_data, right_data) {
    if (left_data) {
        let dst_buffer = audio_buffer.getChannelData(0);
        dst_buffer.set(left_data);
    }

    if (right_data) {
        let dst_buffer = audio_buffer.getChannelData(1);
        dst_buffer.set(right_data);
    }
}
