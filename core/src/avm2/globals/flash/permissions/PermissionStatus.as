package flash.permissions
{
    public final class PermissionStatus
    {
        // Specifies that the permission has been denied.
        public static const DENIED:String = "denied";

        // Specifies that the permission has been granted.
        public static const GRANTED:String = "granted"

        // Specifies that the permission has been granted only when App is in use.
        public static const ONLY_WHEN_IN_USE:String = "onlyWhenInUse"

        // Specifies that the permission hasn't been requested yet.
        // NOTE: On Android, permissionStatus will return UNKNOWN if permission
        // was denied with "Never ask again" option checked
        public static const UNKNOWN:String = "unknown"
    }
}
