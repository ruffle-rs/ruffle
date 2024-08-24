package flash.display
{
  import __ruffle__.stub_getter;
  import flash.geom.Rectangle;

  // [API("733")] Ruffle doesn't support this API Version
  [API(661)]
  public class ScreenMode
  {
    private var _width:int;
    private var _height:int;

    public function ScreenMode(width:int, height:int)
    {
      _width = width;
      _height = height;
    }

    public function get colorDepth():Number
    {
      stub_getter("flash.display.ScreenMode", "colorDepth");
      return 24;
    }

    public function get refreshRate():Number
    {
      stub_getter("flash.display.ScreenMode", "refreshRate");
      return 0;
    }

    public function get height():int
    {
      return _height;
    }

    public function get width():int
    {
      return _width;
    }
  }
}
