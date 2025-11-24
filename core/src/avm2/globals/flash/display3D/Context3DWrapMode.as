package flash.display3D {
    [API("686")]
    public final class Context3DWrapMode {
        public static const CLAMP:String = "clamp";

        [API("696")] // the docs don't mention it, but this is correct
        public static const CLAMP_U_REPEAT_V:String = "clamp_u_repeat_v";

        public static const REPEAT:String = "repeat";

        [API("696")] // the docs don't mention it, but this is correct
        public static const REPEAT_U_CLAMP_V:String = "repeat_u_clamp_v";
    }
}
