/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}




import com.adobe.test.Assert;
// var SECTION = "regress_551587";
// var VERSION = "AS3";
// var TITLE   = "MathClass:_min and _max does not correctly handle -0";
// var bug = "551587";


Assert.expectEq("1.0/Math.min(0.0, 0.0)",
            Infinity,
            1.0/Math.min(0.0, 0.0));

Assert.expectEq("1.0/Math.max(0.0, 0.0)",
            Infinity,
            1.0/Math.max(0.0, 0.0));

Assert.expectEq("1.0/Math.min(0.0, -0.0)",
            -Infinity,
            1.0/Math.min(0.0, -0.0));

Assert.expectEq("1.0/Math.max(-0.0, 0.0)",
            Infinity,
            1.0/Math.max(-0.0, 0.0));

Assert.expectEq("1.0/Math.min(-0.0, -0.0)",
            -Infinity,
            1.0/Math.min(-0.0, -0.0));

Assert.expectEq("1.0/Math.max(-0.0, -0.0)",
            -Infinity,
            1.0/Math.max(-0.0, -0.0));


