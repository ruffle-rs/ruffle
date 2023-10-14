/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;



var o:Object = new Object();
var a:uint = 0x1fffffff;

Assert.expectEq( "o.a = 0x1fffffff, o.b = -1, (o.a == o.b)", false, (o.a = a, o.b = -1, o.a == o.b) );


              // displays results.
