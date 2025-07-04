package flash.system {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [Ruffle(Abstract)]
    public final class Capabilities {
        public static function get avHardwareDisable():Boolean {
            stub_getter("flash.system.Capabilities", "avHardwareDisable");
            return false;
        }

        [API("665")]
        public static function get cpuArchitecture():String {
            stub_getter("flash.system.Capabilities", "cpuArchitecture");
            return "x86";
        }

        public static function get hasAccessibility():Boolean {
            stub_getter("flash.system.Capabilities", "hasAccessibility");
            return false;
        }

        public static function get hasAudio():Boolean {
            stub_getter("flash.system.Capabilities", "hasAudio");
            return true;
        }

        public static function get hasAudioEncoder():Boolean {
            stub_getter("flash.system.Capabilities", "hasAudioEncoder");
            return true;
        }

        public static function get hasEmbeddedVideo():Boolean {
            stub_getter("flash.system.Capabilities", "hasEmbeddedVideo");
            return true;
        }

        public static function get hasIME():Boolean {
            stub_getter("flash.system.Capabilities", "hasIME");
            return true;
        }

        public static function get hasMP3():Boolean {
            stub_getter("flash.system.Capabilities", "hasMP3");
            return true;
        }

        public static function get hasPrinting():Boolean {
            stub_getter("flash.system.Capabilities", "hasPrinting");
            return false;
        }

        public static function get hasScreenBroadcast():Boolean {
            stub_getter("flash.system.Capabilities", "hasScreenBroadcast");
            return false;
        }

        public static function get hasScreenPlayback():Boolean {
            stub_getter("flash.system.Capabilities", "hasScreenPlayback");
            return false;
        }

        public static function get hasStreamingAudio():Boolean {
            stub_getter("flash.system.Capabilities", "hasStreamingAudio");
            return true;
        }

        public static function get hasStreamingVideo():Boolean {
            stub_getter("flash.system.Capabilities", "hasStreamingVideo");
            return true;
        }

        public static function get hasTLS():Boolean {
            stub_getter("flash.system.Capabilities", "hasTLS");
            return true;
        }

        public static function get hasVideoEncoder():Boolean {
            stub_getter("flash.system.Capabilities", "hasVideoEncoder");
            return true;
        }

        public static function get isDebugger():Boolean {
            stub_getter("flash.system.Capabilities", "isDebugger");
            return false;
        }

        public static function get isEmbeddedInAcrobat():Boolean {
            stub_getter("flash.system.Capabilities", "isEmbeddedInAcrobat");
            return false;
        }

        public native static function get language(): String;

        public static function get localFileReadDisable():Boolean {
            stub_getter("flash.system.Capabilities", "localFileReadDisable");
            return false;
        }

        public static function get manufacturer(): String {
            stub_getter("flash.system.Capabilities", "manufacturer");
            return "Adobe Windows";
        }

        [API("662")]
        public static function get maxLevelIDC():String {
            stub_getter("flash.system.Capabilities", "maxLevelIDC");
            return "5.1";
        }

        public native static function get os(): String;

        public native static function get pixelAspectRatio():Number;

        public native static function get playerType(): String;

        public static function get screenColor():String {
            stub_getter("flash.system.Capabilities", "screenColor");
            return "color";
        }

        public native static function get screenDPI():Number;

        public native static function get screenResolutionY():Number;

        public native static function get screenResolutionX():Number;

        public static function get serverString():String {
            stub_getter("flash.system.Capabilities", "serverString");
            return "A=t&SA=t&SV=t&EV=t&MP3=t&AE=t&VE=t&ACC=f&PR=f&SP=t&SB=f&DEB=t&V=WIN%208%2C5%2C0%2C208&M=Adobe%20Windows&R=1600x1200&DP=72&COL=color&AR=1.0&OS=Windows%20XP&L=en&PT=External&AVD=f&LFD=f&WD=f";
        }

        [API("665")]
        public static function get supports32BitProcesses():Boolean {
            stub_getter("flash.system.Capabilities", "supports32BitProcesses");
            return true;
        }

        [API("665")]
        public static function get supports64BitProcesses():Boolean {
            stub_getter("flash.system.Capabilities", "supports64BitProcesses");
            return true;
        }

        [API("667")]
        public static function get touchscreenType():String {
            stub_getter("flash.system.Capabilities", "touchscreenType");
            return "none";
        }

        public native static function get version(): String;

        [API("674")]
        public static function hasMultiChannelAudio(type:String):Boolean {
            stub_method("flash.system.Capabilities", "hasMultiChannelAudio");
            return false;
        }
    }
}
