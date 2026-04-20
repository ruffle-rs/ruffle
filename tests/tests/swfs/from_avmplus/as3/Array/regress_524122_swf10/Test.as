/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "regress_524122";
//     var VERSION = "AS3";
//     var TITLE   = "Array.prototype.sort() on integer properties";
//     var bug     = "524122";

    var testcases = getTestCases();

function getTestCases()
{

    var array = [];
    var item = 0;

    var input = [287000000,-273000000,-270000000,87000000,-245000000];

    // Note, the Array.NUMERIC argument is an ActionScript-ism.
    var actual = input.sort(Array.NUMERIC);

    var expected_old32 = [-273000000,-270000000,87000000,-245000000,287000000];
    var expected_old64 = [87000000,-273000000,-270000000,287000000,-245000000];
    var expected_new = [-273000000,-270000000,-245000000,87000000,287000000];

    var expected;
    if (true) {
        if (actual[0] == expected_old32[0])
            expected = expected_old32;
        else
            expected = expected_old64;
    }
    else
        expected = expected_new;

    array[item++] = Assert.expectEq( "numeric sort", expected.join(), actual.join() );

    return ( array );
}