package flash.display
{
  import flash.geom.Point;
  import flash.geom.Rectangle;
  import flash.events.NativeWindowBoundsEvent;
  import flash.events.Event;
  import flash.events.EventDispatcher;
  import flash.desktop.NativeApplication;
  import __ruffle__.stub_method;
  import __ruffle__.stub_getter;
  import __ruffle__.stub_setter;
  import __ruffle__.stub_constructor;

  [API("661")]
  public class NativeWindow extends EventDispatcher
  {
    public const systemMaxSize:Point = new Point(2880, 2880);
    public const systemMinSize:Point = new Point(1, 1);
    public var minSize:Point = systemMinSize;
    public var maxSize:Point = systemMaxSize;
    public var title:String;
    public var alwaysInFront:Boolean = true;
    public var visible:Boolean = true;

    private var _bounds:Rectangle;
    private var _maximizable:Boolean;
    private var _minimizable:Boolean;
    private var _resizable:Boolean;
    private var _systemChrome:String;
    private var _transparent:Boolean;
    private var _type:String;
    private var _closed:Boolean = false;
    private var _stage:Stage;

    public function NativeWindow(initOptions:NativeWindowInitOptions, _stage:Stage = null)
    {
      stub_constructor("flash.display.NativeWindow");
      NativeApplication.nativeApplication.openedWindows.push(this);
      if (_stage)
      {
        this._stage = _stage;
        _stage.addEventListener(Event.RESIZE, function(e:Event):void
        {
          dispatchEvent(new NativeWindowBoundsEvent(NativeWindowBoundsEvent.RESIZE, false, false, _bounds, _bounds = new Rectangle(x, y, width, height)));
        });
      }

      _maximizable = initOptions.maximizable;
      _minimizable = initOptions.minimizable;
      _resizable = initOptions.resizable;
      _systemChrome = initOptions.systemChrome;
      _transparent = initOptions.transparent;
      _type = initOptions.type;
    }

    public function get width():Number
    {
      stub_getter("flash.display.NativeWindow", "width");
      return _stage.stageWidth;
    }

    public function set width(value:Number):void
    {
      stub_setter("flash.display.NativeWindow", "width");
      _stage.stageWidth = value;
    }

    public function get height():Number
    {
      stub_getter("flash.display.NativeWindow", "height");
      return _stage.stageHeight;
    }

    public function set height(value:Number):void
    {
      stub_setter("flash.display.NativeWindow", "height");
      _stage.stageHeight = value;
    }

    public function get x():Number
    {
      stub_getter("flash.display.NativeWindow", "x");
      return _stage.x;
    }

    public function set x(value:Number):void
    {
      stub_setter("flash.display.NativeWindow", "x");
    }

    public function get y():Number
    {
      stub_getter("flash.display.NativeWindow", "y");
      return _stage.y;
    }

    public function set y(value:Number):void
    {
      stub_setter("flash.display.NativeWindow", "y");
    }

    public function get bounds():Rectangle
    {
      stub_getter("flash.display.NativeWindow", "bounds");
      return _bounds;
    }

    public function set bounds(value:Rectangle):void
    {
      stub_setter("flash.display.NativeWindow", "bounds");
      _bounds = value;
    }

    public function get maximizable():Boolean
    {
      stub_getter("flash.display.NativeWindow", "maximizable");
      return _maximizable;
    }

    public function get minimizable():Boolean
    {
      stub_getter("flash.display.NativeWindow", "minimizable");
      return _minimizable;
    }

    public function get resizable():Boolean
    {
      stub_getter("flash.display.NativeWindow", "resizable");
      return _resizable;
    }

    public function get systemChrome():String
    {
      stub_getter("flash.display.NativeWindow", "systemChrome");
      return _systemChrome;
    }

    public function get transparent():Boolean
    {
      stub_getter("flash.display.NativeWindow", "transparent");
      return _transparent;
    }

    public function get type():String
    {
      stub_getter("flash.display.NativeWindow", "type");
      return _type;
    }

    public function get stage():Stage
    {
      return _stage;
    }

    // Activates this window.
    public function activate():void
    {
      stub_method("flash.display.NativeWindow", "activate");
      dispatchEvent(new Event(Event.ACTIVATE));
    }

    // Closes this window.
    public function close():void
    {
      stub_method("flash.display.NativeWindow", "close");
      if (dispatchEvent(new Event(Event.CLOSING, false, true)))
      {
        _closed = true;
        dispatchEvent(new Event(Event.CLOSE));
        dispatchEvent(new Event(Event.DEACTIVATE));
      }

    }

    // Converts a point in pixel coordinates relative to the origin of the window stage (a global point in terms of the display list), to a point on the virtual desktop.
    public function globalToScreen(globalPoint:Point):Point
    {
      stub_method("flash.display.NativeWindow", "globalToScreen");
      return null;
    }

    // Returns a list of the NativeWindow objects that are owned by this window.
    [API("671")]
    public function listOwnedWindows():Vector.<NativeWindow>
    {
      stub_method("flash.display.NativeWindow", "listOwnedWindows");
      return new Vector.<NativeWindow>();
    }

    // Maximizes this window.
    public function maximize():void
    {
      stub_method("flash.display.NativeWindow", "maximize");
    }

    // Minimizes this window.
    public function minimize():void
    {
      stub_method("flash.display.NativeWindow", "minimize");
    }

    // Triggers a visual cue through the operating system that an event of interest has occurred.
    public function notifyUser(type:String):void
    {
      stub_method("flash.display.NativeWindow", "notifyUser");
    }

    // Sends this window directly behind the specified window.
    public function orderInBackOf(window:NativeWindow):Boolean
    {
      stub_method("flash.display.NativeWindow", "orderInBackOf");
      return false;
    }

    // Brings this window directly in front of the specified window.
    public function orderInFrontOf(window:NativeWindow):Boolean
    {
      stub_method("flash.display.NativeWindow", "orderInFrontOf");
      return false;
    }

    // Sends this window behind any other visible windows.
    public function orderToBack():Boolean
    {
      stub_method("flash.display.NativeWindow", "orderToBack");
      return false;
    }

    // Brings this window in front of any other visible windows.
    public function orderToFront():Boolean
    {
      stub_method("flash.display.NativeWindow", "orderToFront");
      return false;
    }

    // Restores this window from either a minimized or a maximized state.
    public function restore():void
    {
      stub_method("flash.display.NativeWindow", "restore");
    }

    // Starts a system-controlled move of this window.
    public function startMove():Boolean
    {
      stub_method("flash.display.NativeWindow", "startMove");
      return false;
    }

    // Starts a system-controlled resize operation of this window.
    public function startResize(edgeOrCorner:String = "BR"):Boolean
    {
      stub_method("flash.display.NativeWindow", "startResize");
      return false;
    }

    public function get active():Boolean
    {
      stub_getter("flash.display.NativeWindow", "active");
      return true;
    }

    public function get closed():Boolean
    {
      return this._closed;
    }

    public function get displayState():String
    {
      stub_getter("flash.display.NativeWindow", "displayState");
      return "normal";
    }

    [API("668")]
    public function get isSupported():Boolean
    {
      stub_getter("flash.display.NativeWindow", "isSupported");
      return false;
    }

    [API("671")]
    public function get owner():NativeWindow
    {
      stub_getter("flash.display.NativeWindow", "owner");
      return this;
    }

    [API("675")]
    public function get renderMode():String
    {
      stub_getter("flash.display.NativeWindow", "renderMode");
      return "auto";
    }

    public function get supportsMenu():Boolean
    {
      stub_getter("flash.display.NativeWindow", "supportsMenu");
      return false;
    }

    public function get supportsNotification():Boolean
    {
      stub_getter("flash.display.NativeWindow", "supportsNotification");
      return false;
    }

    public function get supportsTransparency():Boolean
    {
      stub_getter("flash.display.NativeWindow", "supportsTransparency");
      return false;
    }
  }
}
