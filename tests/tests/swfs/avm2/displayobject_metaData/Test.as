package {
  import flash.display.Sprite;
  public class Test extends Sprite {
    public function Test() {
      trace("this.metaData: " + this.metaData);
      var obj = {hello: "World"}
      this.metaData = obj;
      trace("this.metaData: " + this.metaData);
      trace("this.metaData === obj: " + (this.metaData === obj));
    }
  }
}
