package flash.net {
    public final class FileFilter {
        private var _description:String;
        private var _extension:String;
        private var _macType:String;
        
        public function FileFilter(description:String, extension:String, macType:String = null) {
            this._description = description;
            this._extension = extension;
            this._macType = macType;
        }

        public function get description(): String {
            return this._description + " (" + this._extension + ")";
        }

        public function set description(val: String): void {
            this._description = val;
        }

        public function get extension(): String {
            return this._extension;
        }

        public function set extension(val: String): void {
            this._extension = val;
        }

        public function get macType(): String {
            return this._macType;
        }

        public function set macType(val: String): void {
            this._macType = val;
        }
        
    }
}