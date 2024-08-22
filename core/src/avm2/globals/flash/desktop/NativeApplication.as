package flash.desktop
{
  import flash.display.NativeWindow;
  import flash.events.InvokeEvent;
  import flash.events.Event;
  import flash.events.TimerEvent;
  import flash.events.EventDispatcher;
  import flash.system.Security;
  import flash.utils.Timer;
  import flash.utils.setTimeout;
  import __ruffle__.stub_method;
  import __ruffle__.stub_getter;
  import __ruffle__.stub_setter;

  [API("661")]
  public final class NativeApplication extends EventDispatcher
  {
    private static var _instance:NativeApplication;

    private var _openedWindows:Array = [];

    private var _idleThreshold:int = 300;

    public function NativeApplication()
    {
      super();
      // TODO
      setTimeout(function():void
      {
        dispatchEvent(new InvokeEvent(InvokeEvent.INVOKE, false, false, null, []));
      }, 500);
    }

    public static function get nativeApplication():NativeApplication
    {
      if (!_instance)
        _instance = new NativeApplication();
      return _instance;
    }

    public static function get supportsMenu():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "supportsMenu");
      return false;
    }

    public static function get supportsDockIcon():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "supportsDockIcon");
      return false;
    }

    public static function get supportsSystemTrayIcon():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "supportsSystemTrayIcon");
      return false;
    }

    [API("668")]
    public static function get supportsDefaultApplication():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "supportsDefaultApplication");
      return false;
    }

    [API("668")]
    public static function get supportsStartAtLogin():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "supportsStartAtLogin");
      return false;
    }

    public function exit(exitCode:int = 0):void
    {
      stub_method("flash.desktop.NativeApplication", "exit");
    }

    public function get runtimeVersion():String
    {
      stub_getter("flash.desktop.NativeApplication", "runtimeVersion");
      return "5.0.0";
    }

    public function get runtimePatchLevel():uint
    {
      stub_getter("flash.desktop.NativeApplication", "runtimePatchLevel");
      return 0;
    }

    public function get applicationID():String
    {
      stub_getter("flash.desktop.NativeApplication", "applicationID");
      return "";
    }

    public function get publisherID():String
    {
      stub_getter("flash.desktop.NativeApplication", "publisherID");
      return "";
    }

    public function get applicationDescriptor():XML
    {
      stub_getter("flash.desktop.NativeApplication", "applicationDescriptor");
      return null;
    }

    public function get autoExit():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "autoExit");
      return false;
    }

    public function set autoExit(param1:Boolean):void
    {
      stub_setter("flash.desktop.NativeApplication", "autoExit");
    }

    public function get icon():InteractiveIcon
    {
      stub_getter("flash.desktop.NativeApplication", "icon");
      return null;
    }

    [API("668")]
    public function get systemIdleMode():String
    {
      stub_getter("flash.desktop.NativeApplication", "systemIdleMode");
      return "normal";
    }

    [API("668")]
    public function set systemIdleMode(param1:String):void
    {
      stub_setter("flash.desktop.NativeApplication", "systemIdleMode");
    }

    public function get startAtLogin():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "startAtLogin");
      return false;
    }

    public function set startAtLogin(param1:Boolean):void
    {
      stub_setter("flash.desktop.NativeApplication", "startAtLogin");
    }

    public function activate(window:NativeWindow = null):void
    {
      stub_method("flash.desktop.NativeApplication", "activate");
    }

    public function get activeWindow():NativeWindow
    {
      stub_getter("flash.desktop.NativeApplication", "activeWindow");
      return _openedWindows[0];
    }

    public function get openedWindows():Array
    {
      stub_getter("flash.desktop.NativeApplication", "openedWindows");
      return _openedWindows;
    }

    public function get timeSinceLastUserInput():int
    {
      stub_getter("flash.desktop.NativeApplication", "timeSinceLastUserInput");
      return 100;
    }

    public function get idleThreshold():int
    {
      stub_getter("flash.desktop.NativeApplication", "idleThreshold");
      return this._idleThreshold;
    }

    public function set idleThreshold(value:int):void
    {
      stub_setter("flash.desktop.NativeApplication", "idleThreshold");
      this._idleThreshold = value;
    }

    public function copy():Boolean
    {
      stub_method("flash.desktop.NativeApplication", "copy");
      return false;
    }

    public function cut():Boolean
    {
      stub_method("flash.desktop.NativeApplication", "cut");
      return false;
    }

    public function paste():Boolean
    {
      stub_method("flash.desktop.NativeApplication", "paste");
      return false;
    }

    public function clear():Boolean
    {
      stub_method("flash.desktop.NativeApplication", "clear");
      return false;
    }

    public function selectAll():Boolean
    {
      stub_method("flash.desktop.NativeApplication", "selectAll");
      return false;
    }

    public function getDefaultApplication(extension:String):String
    {
      stub_method("flash.desktop.NativeApplication", "getDefaultApplication");
      return null;
    }

    public function isSetAsDefaultApplication(extension:String):Boolean
    {
      stub_method("flash.desktop.NativeApplication", "isSetAsDefaultApplication");
      return true;
    }

    public function setAsDefaultApplication(extension:String):void
    {
      stub_method("flash.desktop.NativeApplication", "setAsDefaultApplication");
    }

    public function removeAsDefaultApplication(extension:String):void
    {
      stub_method("flash.desktop.NativeApplication", "removeAsDefaultApplication");
    }

    [API("681")]
    public function get executeInBackground():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "executeInBackground");
      return false;
    }

    [API("681")]
    public function set executeInBackground(param1:Boolean):void
    {
      stub_setter("flash.desktop.NativeApplication", "executeInBackground");
    }

    // [API("721")] Ruffle doesn't support this API Version
    [API("681")]
    public function get isCompiledAOT():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "isCompiledAOT");
      return false;
    }

    // [API("721")] Ruffle doesn't support this API Version
    [API("681")]
    public function get isActive():Boolean
    {
      stub_getter("flash.desktop.NativeApplication", "isActive");
      return true;
    }
  }
}
