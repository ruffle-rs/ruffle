package flash.automation {
    public class Configuration {
        private static var _deviceConfiguration:String = "";

        public static function get testAutomation():Boolean {
            return false;
        }

        public static function get deviceConfiguration():String {
            return _deviceConfiguration;
        }

        public static function set deviceConfiguration(configData:String):void {
            _deviceConfiguration = configData;
        }
    }
}
