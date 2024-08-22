package flash.display
{
  import __ruffle__.stub_getter;
  import flash.geom.Rectangle;

  [API("661")]
  public final class Screen
  {
    private static var _screens:Array;
    private var _width:int;
    private var _height:int;

    [API(661)]
    public var mode:ScreenMode;

    public function Screen(width:int, height:int)
    {
      _width = width;
      _height = height;
      mode = new ScreenMode(width, height);
    }

    public static function get mainScreen():Screen
    {
      return screens[0];
    }

    public static native function getScreensInternal():Array;

    public static function get screens():Array
    {
      if (!_screens)
      {
        _screens = getScreensInternal().map(function(screen:Array, _idx:int, _arr:Array):Screen
        {
          return new Screen(screen[0], screen[1]);
        });
      }
      return _screens;
    }

    public function get colorDepth():Number
    {
      stub_getter("flash.display.Screen", "colorDepth");
      return 24;
    }

    public function get bounds():Rectangle
    {
      return new Rectangle(0, 0, _width, _height);
    }

    public function get visibleBounds():Rectangle
    {
      stub_getter("flash.display.Screen", "visibleBounds");
      return bounds;
    }

    [API(661)]
    public function get modes():Array
    {
      stub_getter("flash.display.Screen", "modes");
      return [];
    }
  }
}