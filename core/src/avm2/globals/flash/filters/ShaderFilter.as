package flash.filters {
    import flash.display.Shader;
    
    public class ShaderFilter extends BitmapFilter {
        private var _shader;
        
        public function ShaderFilter(shader:Shader = null) {
            this._shader = shader;
        }
        
        public function get shader():Shader {
            return this._shader;
        }
        
        public function set shader(value:Shader):void {
            this._shader = value;
        }

        public var bottomExtension:int = 0;
        public var leftExtension:int = 0;
        public var rightExtension:int = 0;
        public var topExtension:int = 0;
        
        // ShaderFilter is the only filter class that doesn't override clone
    }
}
