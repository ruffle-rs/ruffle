package flash.system {
	public final class Security {
		public static native function get pageDomain():String;
		public static native function get sandboxType():String;

		public static native function allowDomain(... domains):void;
		public static native function allowInsecureDomain(... domains):void;
		public static native function loadPolicyFile(url: String):void;
		public static native function showSettings(panel: String = "default"):void;

		[API("661")]
		public static const APPLICATION:String = "application";
		public static const LOCAL_TRUSTED:String = "localTrusted";
		public static const LOCAL_WITH_FILE:String = "localWithFile";
		public static const LOCAL_WITH_NETWORK:String = "localWithNetwork";
		public static const REMOTE:String = "remote";
	}
}
