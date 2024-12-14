package flash.ui {
    // The AS3 docs say this is only available in AIR 3.7.
    // That was determined to be a lie.
    [API("688")]
    public final class GameInputDevice {
        // Specifies the maximum size for the buffer used to cache sampled
        // control values. If `startCachingSamples` returns samples that
        // require more memory than you specify, it throws a memory error.
        public static const MAX_BUFFER_SIZE:int = 32000;
    }
}
