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
 mxmlc -debug test/swfs/avm2/flash/geom/perspectiveprojection/PerspectiveProjectionClass.as
 */

package  {
import flash.display.MovieClip;
import flash.geom.Matrix3D;
import flash.geom.PerspectiveProjection;
import flash.geom.Transform;

public class PerspectiveProjectionClass extends MovieClip {

  public function PerspectiveProjectionClass() {
    var mc:MovieClip = new MovieClip();
    mc.graphics.beginFill(0x0044aa);
    mc.graphics.drawRect(0, 0, 100, 100);
    addChild(mc);
    trace('Initial stage projectionCenter: ' + stage.transform.perspectiveProjection.projectionCenter);

    trace('matrix is initially set: ' + mc.transform.matrix);
    trace('matrix3D is initially null: ' + mc.transform.matrix3D);
    trace(mc.transform.perspectiveProjection);
    trace('perspectiveProjection is initially null: ' + mc.transform.perspectiveProjection);

    var projection:PerspectiveProjection = new PerspectiveProjection();
    trace('Initial fieldOfView: ' + projection.fieldOfView);
    trace('Initial focalLength: ' + projection.focalLength.toFixed(10));
    trace('Initial projectionCenter: ' + projection.projectionCenter);
    mc.transform.perspectiveProjection = projection;
    trace('perspectiveProjection is set: ' + mc.transform.perspectiveProjection);
    trace('matrix is still set: ' + mc.transform.matrix);
    trace('matrix3D is still null: ' + mc.transform.matrix3D);

    var tr: Transform = mc.transform;

    trace('perspectiveProjection returns a clone: ' +
          (tr.perspectiveProjection !== tr.perspectiveProjection));

    projection = root.transform.perspectiveProjection;
    trace('root has a default projection: ' + projection);
    trace('Initial root fieldOfView: ' + projection.fieldOfView);
    trace('Initial root focalLength: ' + projection.focalLength.toFixed(10));
    trace('Initial root projectionCenter: ' + projection.projectionCenter);

    projection.fieldOfView = 100;
    trace('changed root fieldOfView: ' + projection.fieldOfView);
    trace('changed root focalLength: ' + projection.focalLength.toFixed(10));
    projection = root.transform.perspectiveProjection;
    trace('changed root fieldOfView: ' + projection.fieldOfView);
    trace('changed root focalLength: ' + projection.focalLength.toFixed(10));


  }
}

}
