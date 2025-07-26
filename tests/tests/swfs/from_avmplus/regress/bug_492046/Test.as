/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

class State {
    var next = null;
}

const depths = new Vector.<State>(2,true);

function expand(depth)
{
    var x = new State;
    x.next = depths[depth];
    return(x);
}

// var TITLE   = "Regression Testcase for Bug 492046: null value assigned to slot raises assertion failure";


Assert.expectEq("null value assigned to slot should not assert", "[object State]", String(expand(1)) );



