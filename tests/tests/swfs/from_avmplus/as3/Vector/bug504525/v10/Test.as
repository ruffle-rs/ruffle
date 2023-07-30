/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "Vector";
//     var VERSION = "as3";
//     var TITLE   = "bug 504525";


function vtest()
{
    var v1 = new Vector.<int>(); v1.push( 1 );
    var v2 = new Vector.<int>(); v2.push( 2 );
    var v3 = new Vector.<int>(); v3.push( 3 );
    var v4 = new Vector.<int>(); v4.push( 4 );

    return v1.concat( v2, v3, v4 );
}

var result = vtest();

Assert.expectEq("Test Vector.concat with SWF 9 or 10 behavior",
  "1,4,3,2",
  result.toString());
