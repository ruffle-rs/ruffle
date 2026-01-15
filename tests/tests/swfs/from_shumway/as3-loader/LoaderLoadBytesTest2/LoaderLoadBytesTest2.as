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
 * mxmlc test/swfs/as3-loader/LoaderLoadBytesTest2.as -debug -output test/swfs/as3-loader/LoaderLoadBytesTest2.swf
 */
package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.ProgressEvent;
import flash.system.fscommand;
import flash.utils.ByteArray;

public class LoaderLoadBytesTest2 extends Sprite {
  private var _loader:Loader;
  private var _progressDispatched: Boolean;

  public function LoaderLoadBytesTest2() {
    _loader = new Loader();
    _loader.contentLoaderInfo.addEventListener(Event.OPEN, loader_open);
    _loader.contentLoaderInfo.addEventListener(ProgressEvent.PROGRESS, loader_progress);
    _loader.contentLoaderInfo.addEventListener(Event.INIT, loader_init);
    _loader.contentLoaderInfo.addEventListener(Event.COMPLETE, loader_complete);
    addChild(_loader);

    var arr: Array = [137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,1,0,0,0,1,8,6,0,0,0,31,21,196,137,0,0,0,13,73,68,65,84,8,153,99,248,207,192,240,31,0,5,0,1,255,171,206,54,137,0,0,0,0,73,69,78,68,174,66,96,130];
    var bytes: ByteArray = new ByteArray();
    for (var i: int = 0; i < arr.length; i++) bytes.writeByte(arr[i]);
    bytes.position = 0;

    _loader.loadBytes(bytes);
  }
  private function loader_open(event:Event):void {
    trace("TODO: don't dispatch OPEN for loadBytes-loaded content");
  }

  private function loader_progress(event:ProgressEvent):void {
    if (_progressDispatched) {
      return;
    }
    _progressDispatched = true;
    trace("loading progress " + event.bytesLoaded + ' ' + event.bytesTotal +
          ' TODO: make sure this is dispatched twice before content is initialized');
  }

  private function loader_init(event:Event):void {
    var url:String = _loader.contentLoaderInfo.url;
    url = url.substr(url.lastIndexOf('.swf') + 4);
    trace("loadee initialized, url relative to swf: " + url);
  }

  private function loader_complete(event:Event):void {
    trace("loading of " + _loader.contentLoaderInfo.bytesLoaded + " bytes complete");
    fscommand('quit');
  }
}
}
