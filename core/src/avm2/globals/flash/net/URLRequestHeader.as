package flash.net {
    public final class URLRequestHeader {
        [Ruffle(NativeAccessible)]
        public var name:String;

        [Ruffle(NativeAccessible)]
        public var value:String;

        public function URLRequestHeader(name:String = "", value:String = "") {
            this.name = name;
            this.value = value;
        }
    }
}
