package {
    import com.very.long.namespace.example;
    import flash.display.Sprite;
    import flash.utils.ByteArray;

    import flash.display.MovieClip;

    public class Test extends MovieClip {
        namespace avmplus = "avmplus";
        namespace flash_utils = "flash.utils";

        public function Test() {
            var functions:Array = [avmplus::getQualifiedSuperclassName, flash_utils::getQualifiedSuperclassName];
            for (var i = 0; i < 2; i ++) {
                var currentFunction:Function = functions[i];

                trace(currentFunction(Test));
                trace(currentFunction(ByteArray));
                trace(currentFunction(example));
                trace(currentFunction(int));
                trace(currentFunction(String));
                trace(currentFunction(flash.display.Sprite));
                trace(currentFunction(null));
                trace(currentFunction(undefined));
                trace(currentFunction(478));
            }
        }
    }
}
