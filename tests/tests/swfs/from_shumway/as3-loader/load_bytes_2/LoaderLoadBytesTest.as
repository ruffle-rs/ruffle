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
 * mxmlc test/swfs/as3-loader/LoaderLoadBytesTest.as -debug -output test/swfs/as3-loader/LoaderLoadBytesTest.swf
 */
package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.ProgressEvent;
import flash.system.fscommand;

public class LoaderLoadBytesTest extends Sprite {
  [Embed(source='Loadee.swf', mimeType='application/octet-stream')]
  private static const BYTES:Class;

  private var _loader:Loader;
  private var _progressDispatched: Boolean;

  public function LoaderLoadBytesTest() {
    _loader = new Loader();
    _loader.contentLoaderInfo.addEventListener(Event.OPEN, loader_open);
    _loader.contentLoaderInfo.addEventListener(ProgressEvent.PROGRESS, loader_progress);
    _loader.contentLoaderInfo.addEventListener(Event.INIT, loader_init);
    _loader.contentLoaderInfo.addEventListener(Event.COMPLETE, loader_complete);
    addChild(_loader);
    _loader.loadBytes(new BYTES());
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
