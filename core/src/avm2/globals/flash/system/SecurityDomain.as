package flash.system {
    public class SecurityDomain {
        private static var dummyDomain : SecurityDomain = new SecurityDomain();
        public static function get currentDomain() :  SecurityDomain {
            return dummyDomain;
        }
    }
}
