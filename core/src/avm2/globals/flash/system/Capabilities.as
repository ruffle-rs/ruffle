package flash.system {

    [Ruffle(Abstract)]
    public final class Capabilities {
        public static function get avHardwareDisable():Boolean {
            return false;
        }

        [API("665")]
        public static function get cpuArchitecture():String {
            return "x86";
        }

        public static function get hasAccessibility():Boolean {
            return false;
        }

        public static function get hasAudio():Boolean {
            return true;
        }

        public static function get hasAudioEncoder():Boolean {
            return true;
        }

        public static function get hasEmbeddedVideo():Boolean {
            return true;
        }

        public static function get hasIME():Boolean {
            return true;
        }

        public static function get hasMP3():Boolean {
            return true;
        }

        public static function get hasPrinting():Boolean {
            return false;
        }

        public static function get hasScreenBroadcast():Boolean {
            return false;
        }

        public static function get hasScreenPlayback():Boolean {
            return false;
        }

        public static function get hasStreamingAudio():Boolean {
            return true;
        }

        public static function get hasStreamingVideo():Boolean {
            return true;
        }

        public static function get hasTLS():Boolean {
            return true;
        }

        public static function get hasVideoEncoder():Boolean {
            return true;
        }

        public static function get isDebugger():Boolean {
            return false;
        }

        public static function get isEmbeddedInAcrobat():Boolean {
            return false;
        }

        public native static function get language():String;

        public static function get localFileReadDisable():Boolean {
            return false;
        }

        public static function get manufacturer():String {
            return "Adobe Windows";
        }

        [API("662")]
        public static function get maxLevelIDC():String {
            return "5.1";
        }

        public native static function get os():String;

        public native static function get pixelAspectRatio():Number;

        public native static function get playerType():String;

        public static function get screenColor():String {
            return "color";
        }

        public native static function get screenDPI():Number;

        public native static function get screenResolutionY():Number;

        public native static function get screenResolutionX():Number;

        public static function get serverString():String {
            return "A=t&SA=t&SV=t&EV=t&MP3=t&AE=t&VE=t&ACC=f&PR=f&SP=t&SB=f&DEB=t&V=WIN%208%2C5%2C0%2C208&M=Adobe%20Windows&R=1600x1200&DP=72&COL=color&AR=1.0&OS=Windows%20XP&L=en&PT=External&AVD=f&LFD=f&WD=f";
        }

        [API("665")]
        public static function get supports32BitProcesses():Boolean {
            return true;
        }

        [API("665")]
        public static function get supports64BitProcesses():Boolean {
            return true;
        }

        [API("667")]
        public static function get touchscreenType():String {
            return "none";
        }

        public native static function get version():String;

        [API("674")]
        public static function hasMultiChannelAudio(type:String):Boolean {
            return false;
        }
    }
}
