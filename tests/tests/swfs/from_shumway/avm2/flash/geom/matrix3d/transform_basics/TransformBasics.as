/*
 * Copyright 2015 Mozilla Foundation
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

/*
 Compiled with:
 mxmlc -debug test/swfs/avm2/flash/geom/matrix3d/TransformBasics.as
 */

package  {
import flash.display.MovieClip;
import flash.geom.Matrix3D;


public class TransformBasics extends MovieClip {

  public function TransformBasics() {
    var mc:MovieClip = new MovieClip();
    mc.graphics.beginFill(0x0044aa);
    mc.graphics.drawRect(0, 0, 100, 100);
    addChild(mc);
    trace('matrix is initially set: ' + mc.transform.matrix);
    trace('matrix3D is initially null: ' + mc.transform.matrix3D);
    var m: Matrix3D = new Matrix3D();
    mc.transform.matrix3D = m;
    trace('matrix3D is set: ' + mc.transform.matrix3D);
    trace('Setting matrix3D nulls matrix: ' + mc.transform.matrix);
    trace('Matrix3D#rawData returns clone: ' + (m.rawData !== m.rawData));
    trace('Matrix3D#rawData has initial values: ' + m.rawData);
    m.rawData[13] = 100;
    trace('Assigning to rawData fields does nothing: ' + m.rawData);
    var mc2:MovieClip = new MovieClip();
    try {
      mc2.transform.matrix3D = m;
    } catch(e: Error) {
      trace("Assigning same Matrix3D instance to two DisplayObject's transforms throws:");
      trace('' + e);
    }
    try {
      trace("Calling getRelativeMatrix3D with null target throws:");
      mc2.transform.getRelativeMatrix3D(null);
    } catch(e: Error) {
      trace('' + e);
    }
    trace('Transform#getRelativeMatrix3D returns null if no Matrix3D is set: ' + mc2.transform.getRelativeMatrix3D(stage));
    trace('Transform#getRelativeMatrix3D returns Matrix3D if Matrix3D is set: ' + mc.transform.getRelativeMatrix3D(stage));
  }
}

}
