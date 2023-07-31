/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}


import UintVarAsClassProp.*;
import com.adobe.test.Assert;



var obj = new testuint();

Assert.expectEq( "Uint public property", 1, obj.num1 );


              // displays results.
