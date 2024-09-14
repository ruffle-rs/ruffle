/*
 * Copyright 2014 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Compiled with:
 * java -jar utils/asc.jar -import build/playerglobal/playerglobal-single.abc -swf AVM1MovieTest,600,600 test/swfs/avm1movie/AVM1MovieTest.as
 */
package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.MouseEvent;
import flash.geom.Point;
import flash.net.URLRequest;
import flash.system.fscommand;

public class AVM1MovieTest extends Sprite {

  public function AVM1MovieTest() {
    var basePath:String = stage.loaderInfo.url;
    basePath = basePath.split(/\?#/)[0];
    var pathParts:Array = basePath.split('/');
    pathParts[pathParts.length - 1] = '';
    basePath = pathParts.join('/');
    _loader = new Loader();
    _loader.name = 'loader';
    _loader.contentLoaderInfo.addEventListener(Event.OPEN, loader_open);
    _loader.contentLoaderInfo.addEventListener(Event.INIT, loader_init);
    _loader.contentLoaderInfo.addEventListener(Event.COMPLETE, loader_complete);
    _loader.addEventListener(MouseEvent.CLICK, loader_click);
    addChild(_loader);
    _loader.load(new URLRequest(basePath + "avm1-loadee.swf"));
  }
  private var _loader:Loader;
  private function loader_open(event:Event):void {
    trace("loading started");
  }

  private function loader_init(event:Event):void {
    trace("loadee initialized");
  }

  private function loader_complete(event:Event):void {
    trace("loading complete");
    trace('bytes loaded: ' + _loader.contentLoaderInfo.bytesLoaded);
    trace('under point: ' + this.getObjectsUnderPoint(new Point(10, 10))[0]);
    trace('bounds: ' + _loader.content.getBounds(this));
    trace('width: ' + _loader.content.width);
    trace('bbox  hit test: ' + _loader.content.hitTestPoint(10, 10, false));
    trace('shape hit test: ' + _loader.content.hitTestPoint(10, 10, true));
    trace('bbox  hit test: ' + _loader.content.hitTestPoint(105, 10, false));
//            fscommand('quit');
  }

  private function loader_click(event:Event):void {
    trace("loader clicked");
    trace('target: ' + event.target.name);
  }
}
}
