/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import bug127174.*;
import com.adobe.test.Assert;


var t:IFaceCorceErrorTest = new IFaceCorceErrorTest();
var k:I2TestInterface = new IFaceCorceErrorZ();
Assert.expectEq("regress for bug 127714", true, t.causeCoerceError(k));


