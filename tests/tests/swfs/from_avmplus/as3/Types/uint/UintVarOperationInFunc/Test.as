/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;





function UintArgs(n1:uint , n2:uint):uint {
 
 var n3:uint;
 
 n3 = n1 + n2;
 
 n3 = n3 - 10;
 
 n3 = n3 * 10;
 
 n3 = n3 / 10;
 
 return n3;

}


Assert.expectEq( "Calling function with 1 uint argument", 20 , UintArgs(20,10) );


              // displays results.
