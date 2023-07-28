/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 Description:  The elements of this object are converted to strings and
 these strings are then concatenated, separated by comma
 characters. The result is the same as if the built-in join
 method were invoiked for this object with no argument.
 */

// var SECTION = "15.4.4.3-1";
// var VERSION = "ECMA_1";


var v1=new Vector.<int>();
Assert.expectEq(    "join empty vector",
  "",
  v1.join());

var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=i;
Assert.expectEq(    "join vector 0-9",
  "0,1,2,3,4,5,6,7,8,9",
  v1.join());

var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=i;
Assert.expectEq(    "join vector with 'and' separator",
  "0 and 1 and 2 and 3 and 4 and 5 and 6 and 7 and 8 and 9",
  v1.join(" and "));

var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=i;
Assert.expectEq(    "join vector with '|' separator",
  "0|1|2|3|4|5|6|7|8|9",
  v1.join("|"));
