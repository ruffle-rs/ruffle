/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/**
 Description:
 Runtime specialization
 Specialization of the Vector type can be done at runtime
 instead of at compile time.
 */

function getVector() { return Vector; }

var CODE = 1007; //  Instantiation attempted on a non-constructor.

// var TITLE="Runtime specialization";

var x = getVector().<int>;
y = new x();

y.push(1);
y.push(2);
y.push(3);

Assert.expectEq(    "Vector constructed via runtime specialization",
  3,
  y.length);

Assert.expectEq(    "Vector constructed via runtime specialization",
  2,
  y[1]);


var neg_result = "Failed to catch invalid construct";

try {
  unspecialized = getVector();
  var z = new unspecialized();
} catch ( ex )
{
  neg_result = String(ex);
}

Assert.expectEq ( "Invalid use of unspecialized type in constructor", Utils.TYPEERROR + CODE,  Utils.typeError(neg_result));
