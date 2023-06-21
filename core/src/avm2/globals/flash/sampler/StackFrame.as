package flash.sampler {
    public final class StackFrame {
        public const name:String;

        public const file:String;

        public const line:uint;
   
        public const scriptID:Number;
      
        public function toString():String {
            if (this.file) {
                return this.name + "()[" + this.file + ":" + this.line + "]";
            } else {
                return this.name + "()";
            }
        }
    }
}

