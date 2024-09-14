package {
  import flash.display.Sprite;
  public class Test extends Sprite {
    public function Test() {
      trace("// Object.prototype.toString.call(new Vector.<*>())");
      trace(Object.prototype.toString.call(new Vector.<*>()));

      trace("// Object.prototype.toString.call(new Vector.<Object>())");
      trace(Object.prototype.toString.call(new Vector.<Object>()));

      trace("// Object.prototype.toString.call(new Vector.<String>())");
      trace(Object.prototype.toString.call(new Vector.<String>()));

      trace("// Object.prototype.toString.call(new Vector.<XML>())");
      trace(Object.prototype.toString.call(new Vector.<XML>()));

      trace("// Object.prototype.toString.call(new Vector.<Sprite>())");
      trace(Object.prototype.toString.call(new Vector.<Sprite>()));
    }
  }
}
