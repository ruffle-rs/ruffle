package flash.security {
    [API("672")]
    public final class CertificateStatus {
        public static const EXPIRED:String = "expired";
        public static const INVALID:String = "invalid";
        public static const INVALID_CHAIN:String = "invalidChain";
        public static const NOT_YET_VALID:String = "notYetValid";
        public static const PRINCIPAL_MISMATCH:String = "principalMismatch";
        public static const REVOKED:String = "revoked";
        public static const TRUSTED:String = "trusted";
        public static const UNKNOWN:String = "unknown";
        public static const UNTRUSTED_SIGNERS:String = "untrustedSigners";
    }
}
